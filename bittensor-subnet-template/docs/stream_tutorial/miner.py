import copy
import time
import asyncio
import argparse
import threading
import traceback
from abc import ABC, abstractmethod
from functools import partial
from starlette.types import Send

import bittensor as bt
from transformers import GPT2Tokenizer
from typing import List, Dict, Tuple, Union, Callable, Awaitable

from protocol import StreamPrompting
from config import get_config, check_config


class StreamMiner(ABC):
    def __init__(self, config=None, axon=None, wallet=None, subtensor=None):
        # Setup base config from Miner.config() and merge with subclassed config.
        base_config = copy.deepcopy(config or get_config())
        self.config = self.config()
        self.config.merge(base_config)

        check_config(StreamMiner, self.config)
        bt.logging.info(self.config)  # TODO: duplicate print?

        self.prompt_cache: Dict[str, Tuple[str, int]] = {}

        # Activating Bittensor's logging with the set configurations.
        bt.logging(config=self.config, logging_dir=self.config.full_path)
        bt.logging.info("Setting up bittensor objects.")

        # Wallet holds cryptographic information, ensuring secure transactions and communication.
        self.wallet = wallet or bt.wallet(config=self.config)
        bt.logging.info(f"Wallet {self.wallet}")

        # subtensor manages the blockchain connection, facilitating interaction with the Bittensor blockchain.
        self.subtensor = subtensor or bt.subtensor(config=self.config)
        bt.logging.info(f"Subtensor: {self.subtensor}")
        bt.logging.info(
            f"Running miner for subnet: {self.config.netuid} on network: {self.subtensor.chain_endpoint} with config:"
        )

        # metagraph provides the network's current state, holding state about other participants in a subnet.
        self.metagraph = self.subtensor.metagraph(self.config.netuid)
        bt.logging.info(f"Metagraph: {self.metagraph}")

        if self.wallet.hotkey.ss58_address not in self.metagraph.hotkeys:
            bt.logging.error(
                f"\nYour validator: {self.wallet} if not registered to chain connection: {self.subtensor} \nRun btcli register and try again. "
            )
            exit()
        else:
            # Each miner gets a unique identity (UID) in the network for differentiation.
            self.my_subnet_uid = self.metagraph.hotkeys.index(
                self.wallet.hotkey.ss58_address
            )
            bt.logging.info(f"Running miner on uid: {self.my_subnet_uid}")

        # The axon handles request processing, allowing validators to send this process requests.
        self.axon = axon or bt.axon(
            wallet=self.wallet, port=self.config.axon.port
        )
        # Attach determiners which functions are called when servicing a request.
        bt.logging.info(f"Attaching forward function to axon.")
        print(f"Attaching forward function to axon. {self._prompt}")
        self.axon.attach(
            forward_fn=self._prompt,
        )
        bt.logging.info(f"Axon created: {self.axon}")

        # Instantiate runners
        self.should_exit: bool = False
        self.is_running: bool = False
        self.thread: threading.Thread = None
        self.lock = asyncio.Lock()
        self.request_timestamps: Dict = {}

    @abstractmethod
    def config(self) -> "bt.Config":
        ...

    @classmethod
    @abstractmethod
    def add_args(cls, parser: argparse.ArgumentParser):
        ...

    def _prompt(self, synapse: StreamPrompting) -> StreamPrompting:
        """
        A wrapper method around the `prompt` method that will be defined by the subclass.

        This method acts as an intermediary layer to perform pre-processing before calling the
        actual `prompt` method implemented in the subclass. Specifically, it checks whether a
        prompt is in cache to avoid reprocessing recent requests. If the prompt is not in the
        cache, the subclass `prompt` method is called.

        Args:
            synapse (StreamPrompting): The incoming request object encapsulating the details of the request.

        Returns:
            StreamPrompting: The response object to be sent back in reply to the incoming request, essentially
            the filled synapse request object.

        Raises:
            ValueError: If the prompt is found in the cache indicating it was sent recently.

        Example:
            This method is not meant to be called directly but is invoked internally when a request
            is received, and it subsequently calls the `prompt` method of the subclass.
        """
        return self.prompt(synapse)

    @abstractmethod
    def prompt(self, synapse: StreamPrompting) -> StreamPrompting:
        """
        Abstract method to handle and respond to incoming requests to the miner.

        Subclasses should implement this method to define their custom logic for processing and
        responding to requests. This method is designed to be overridden, and its behavior will
        be dependent on the specific implementation provided in the subclass.

        Args:
            synapse (StreamPrompting): The incoming request object encapsulating the details
                of the request. This must contain `messages` and `roles` as fields.

        Returns:
            StreamPrompting: The response object that should be sent back in reply to the
                incoming request. This is essentially the filled synapse request object.

        Example:
            class CustomMiner(Miner):
                def prompt(self, synapse: StreamPrompting) -> StreamPrompting:
                    # Custom logic to process and respond to the request.
                    synapse.completion = "The meaning of life is 42."
                    return synapse
        """
        ...

    def run(self):
        """
        Runs the miner logic. This method starts the miner's operations, including
        listening for incoming requests and periodically updating the miner's knowledge
        of the network graph.
        """
        if not self.subtensor.is_hotkey_registered(
            netuid=self.config.netuid,
            hotkey_ss58=self.wallet.hotkey.ss58_address,
        ):
            bt.logging.error(
                f"Wallet: {self.wallet} is not registered on netuid {self.config.netuid}"
                f"Please register the hotkey using `btcli subnets register` before trying again"
            )
            exit()

        # Serve passes the axon information to the network + netuid we are hosting on.
        # This will auto-update if the axon port of external ip have changed.
        bt.logging.info(
            f"Serving axon {StreamPrompting} on network: {self.config.subtensor.chain_endpoint} with netuid: {self.config.netuid}"
        )
        self.axon.serve(netuid=self.config.netuid, subtensor=self.subtensor)

        # Start  starts the miner's axon, making it active on the network.
        bt.logging.info(
            f"Starting axon server on port: {self.config.axon.port}"
        )
        self.axon.start()

        # --- Run until should_exit = True.
        self.last_epoch_block = self.subtensor.get_current_block()
        bt.logging.info(f"Miner starting at block: {self.last_epoch_block}")

        # This loop maintains the miner's operations until intentionally stopped.
        bt.logging.info(f"Starting main loop")
        step = 0
        try:
            while not self.should_exit:
                start_epoch = time.time()

                # --- Wait until next epoch.
                current_block = self.subtensor.get_current_block()
                while (
                    current_block - self.last_epoch_block
                    < self.config.miner.blocks_per_epoch
                ):
                    # --- Wait for next bloc.
                    time.sleep(1)
                    current_block = self.subtensor.get_current_block()

                    # --- Check if we should exit.
                    if self.should_exit:
                        break

                # --- Update the metagraph with the latest network state.
                self.last_epoch_block = self.subtensor.get_current_block()

                metagraph = self.subtensor.metagraph(
                    netuid=self.config.netuid,
                    lite=True,
                    block=self.last_epoch_block,
                )
                log = (
                    f"Step:{step} | "
                    f"Block:{metagraph.block.item()} | "
                    f"Stake:{metagraph.S[self.my_subnet_uid]} | "
                    f"Rank:{metagraph.R[self.my_subnet_uid]} | "
                    f"Trust:{metagraph.T[self.my_subnet_uid]} | "
                    f"Consensus:{metagraph.C[self.my_subnet_uid] } | "
                    f"Incentive:{metagraph.I[self.my_subnet_uid]} | "
                    f"Emission:{metagraph.E[self.my_subnet_uid]}"
                )
                bt.logging.info(log)

                step += 1

        # If someone intentionally stops the miner, it'll safely terminate operations.
        except KeyboardInterrupt:
            self.axon.stop()
            bt.logging.success("Miner killed by keyboard interrupt.")
            exit()

        # In case of unforeseen errors, the miner will log the error and continue operations.
        except Exception as e:
            bt.logging.error(traceback.format_exc())

    def run_in_background_thread(self):
        """
        Starts the miner's operations in a separate background thread.
        This is useful for non-blocking operations.
        """
        if not self.is_running:
            bt.logging.debug("Starting miner in background thread.")
            self.should_exit = False
            self.thread = threading.Thread(target=self.run, daemon=True)
            self.thread.start()
            self.is_running = True
            bt.logging.debug("Started")

    def stop_run_thread(self):
        """
        Stops the miner's operations that are running in the background thread.
        """
        if self.is_running:
            bt.logging.debug("Stopping miner in background thread.")
            self.should_exit = True
            self.thread.join(5)
            self.is_running = False
            bt.logging.debug("Stopped")

    def __enter__(self):
        """
        Starts the miner's operations in a background thread upon entering the context.
        This method facilitates the use of the miner in a 'with' statement.
        """
        self.run_in_background_thread()

    def __exit__(self, exc_type, exc_value, traceback):
        """
        Stops the miner's background operations upon exiting the context.
        This method facilitates the use of the miner in a 'with' statement.

        Args:
            exc_type: The type of the exception that caused the context to be exited.
                      None if the context was exited without an exception.
            exc_value: The instance of the exception that caused the context to be exited.
                       None if the context was exited without an exception.
            traceback: A traceback object encoding the stack trace.
                       None if the context was exited without an exception.
        """
        self.stop_run_thread()


class StreamingTemplateMiner(StreamMiner):
    def config(self) -> "bt.Config":
        """
        Returns the configuration object specific to this miner.

        Implement and extend this method to provide custom configurations for the miner.
        Currently, it sets up a basic configuration parser.

        Returns:
            bt.Config: A configuration object with the miner's operational parameters.
        """
        parser = argparse.ArgumentParser(description="Streaming Miner Configs")
        self.add_args(parser)
        return bt.config(parser)

    def add_args(cls, parser: argparse.ArgumentParser):
        """
        Adds custom arguments to the command line parser.

        Developers can introduce additional command-line arguments specific to the miner's
        functionality in this method. These arguments can then be used to configure the miner's operation.

        Args:
            parser (argparse.ArgumentParser):
                The command line argument parser to which custom arguments should be added.
        """
        pass

    def prompt(self, synapse: StreamPrompting) -> StreamPrompting:
        """
        Generates a streaming response for the provided synapse.

        This function serves as the main entry point for handling streaming prompts. It takes
        the incoming synapse which contains messages to be processed and returns a streaming
        response. The function uses the GPT-2 tokenizer and a simulated model to tokenize and decode
        the incoming message, and then sends the response back to the client token by token.

        Args:
            synapse (StreamPrompting): The incoming StreamPrompting instance containing the messages to be processed.

        Returns:
            StreamPrompting: The streaming response object which can be used by other functions to
                            stream back the response to the client.

        Usage:
            This function can be extended and customized based on specific requirements of the
            miner. Developers can swap out the tokenizer, model, or adjust how streaming responses
            are generated to suit their specific applications.
        """
        bt.logging.trace("HI. PROMPT()")
        tokenizer = GPT2Tokenizer.from_pretrained("gpt2")

        # Simulated function to decode token IDs into strings. In a real-world scenario,
        # this can be replaced with an actual model inference step.
        def model(ids):
            return (tokenizer.decode(id) for id in ids)

        async def _prompt(text: str, send: Send):
            """
            Asynchronously processes the input text and sends back tokens as a streaming response.

            This function takes an input text, tokenizes it using the GPT-2 tokenizer, and then
            uses the simulated model to decode token IDs into strings. It then sends each token
            back to the client as a streaming response, with a delay between tokens to simulate
            the effect of real-time streaming.

            Args:
                text (str): The input text message to be processed.
                send (Send): An asynchronous function that allows sending back the streaming response.

            Usage:
                This function can be adjusted based on the streaming requirements, speed of
                response, or the model being used. Developers can also introduce more sophisticated
                processing steps or modify how tokens are sent back to the client.
            """
            bt.logging.trace("HI. _PROMPT()")
            input_ids = tokenizer(
                text, return_tensors="pt"
            ).input_ids.squeeze()
            buffer = []
            bt.logging.debug(f"Input text: {text}")
            bt.logging.debug(f"Input ids: {input_ids}")

            N = 3  # Number of tokens to send back to the client at a time
            for token in model(input_ids):
                bt.logging.trace(f"appending token: {token}")
                buffer.append(token)
                # If buffer has N tokens, send them back to the client.
                if len(buffer) == N:
                    time.sleep(0.1)
                    joined_buffer = "".join(buffer)
                    bt.logging.debug(f"sedning tokens: {joined_buffer}")
                    await send(
                        {
                            "type": "http.response.body",
                            "body": joined_buffer.encode("utf-8"),
                            "more_body": True,
                        }
                    )
                    bt.logging.debug(f"Streamed tokens: {joined_buffer}")
                    buffer = []  # Clear the buffer for next batch of tokens

            # Send any remaining tokens in the buffer
            if buffer:
                joined_buffer = "".join(buffer)
                await send(
                    {
                        "type": "http.response.body",
                        "body": joined_buffer.encode("utf-8"),
                        "more_body": False,  # No more tokens to send
                    }
                )
                bt.logging.trace(f"Streamed tokens: {joined_buffer}")

        message = synapse.messages[0]
        bt.logging.trace(f"message in _prompt: {message}")
        token_streamer = partial(_prompt, message)
        bt.logging.trace(f"token streamer: {token_streamer}")
        return synapse.create_streaming_response(token_streamer)


# This is the main function, which runs the miner.
if __name__ == "__main__":
    with StreamingTemplateMiner():
        while True:
            time.sleep(1)
