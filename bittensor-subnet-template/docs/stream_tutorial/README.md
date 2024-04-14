# Bittensor Streaming Tutorial
This document is intented as a developer-friendly walkthrough of integrating streaming into your bittensor application.

If you prefer to jump right into a complete stand-alone example, see:
- `miner.py`
- `protocol.py`
- `client.py`

Start your miner:
```bash
python miner.py --netuid 8 --wallet.name default --wallet.hotkey miner --subtensor.network test --axon.port 10000 --logging.trace
```

Run the client:
```bash
python client.py --netuid 8 --my_uid 1 --network test
```

## Overview
This tutorial is designed to show you how to use the streaming API to integrate into your application. It will cover the following topics:
- writing your streaming protocol (inherits from bittensor.StreamingSynapse)
- writing your streaming server (uses your streaming protocol)
- writing your streaming client (uses your streaming protocol)

### Defining your streaming protocol
When designing your protocol, it would be helpful to look at the bittensor.StreamingSynapse for reference. Below is a condensed snippet of the abstract methods that you will need to implement in your subclass.

You will need to implement two methods:

- `process_streaming_response`
- `extract_response_json`

These two methods are the core of your streaming protocol. The first method process_streaming_response is called as the response is being streamed from the network. It is responsible for handling the streaming response, such as parsing and accumulating data. The second method extract_response_json is  called after the response has been processed and is responsible for retrieving structured data to be post-processed in the dendrite in bittensor core code.

```python
class StreamingSynapse(bittensor.Synapse, ABC):
    ...
    class BTStreamingResponse(_StreamingResponse):
        ...
    @abstractmethod
    async def process_streaming_response(self, response: Response):
        """
        Abstract method that must be implemented by the subclass.
        This method should provide logic to handle the streaming response, such as parsing and accumulating data.
        It is called as the response is being streamed from the network, and should be implemented to handle the specific
        streaming data format and requirements of the subclass.

        Args:
            response: The response object to be processed, typically containing chunks of data.
        """
        ...

    @abstractmethod
    def extract_response_json(self, response: Response) -> dict:
        """
        Abstract method that must be implemented by the subclass.
        This method should provide logic to extract JSON data from the response, including headers and content.
        It is called after the response has been processed and is responsible for retrieving structured data
        that can be used by the application.

        Args:
            response: The response object from which to extract JSON data.
        """
        ...
    ...
```

See the full reference code at the bittensor [repo](https://github.com/opentensor/bittensor/blob/master/bittensor/stream.py).


#### Create your protocol
Let's walk through how to create a protocol using the bittensor.StreamingSynapse class.
```python
class MyStreamingSynapse(bt.StreamingSynapse):
    # define your expected data fields here as pydantic field objects
    # This allows you to control what information is passed along the network
    messages: List[str] = pydantic.Field(
        ..., # this ellipsis (...) indicates the object is required
        title="Messages", # What is the name of this field?
        description="A list of messages in the Prompting scenario. Immutable.",
        allow_mutation=False, # disallow modification of this field after creation
    )
    completion: str = pydantic.Field(
        "",
        title="Completion",
    )
    # add fields as necessary
    ...

    # This method controls how your synapse is deserialized from the network
    # E.g. you can extract whatever information you want to receive at the final
    # yield in the async generator returned by the server, without receiving
    # the entire synapse object itself.
    # In this example, we just want the completion string at the end.
    def deserialize(self) -> str:
        return self.completion

    # implement your `process_streaming_response` logic to actually yield objects to the streamer
    # this effectively defines the async generator that you'll recieve on the client side
    async def process_streaming_response(self, response: MyStreamingSynapse):
        # this is an example of how you might process a streaming response
        # iterate over the response content and yield each line
        async for chunk in response.content.iter_any():
            tokens = chunk.decode("utf-8").split("\n")
            yield tokens
    
    # implement `extract_response_json` to extract the JSON data from the response headers
    # this will be dependent on the data you are streaming and how you want to structure it
    # it MUST conform to the following format expected by the bittensor dendrite:
    """
        {
            # METADATA AND HEADERS
            "name": ...,
            "timeout": float(...),
            "total_size": int(...),
            "header_size": int(...),
            "dendrite": ...,
            "axon": ...,
            # YOUR FIELDS
            "messages": self.messages,
            ...
        }
    """
    def extract_response_json(self, response: MyStreamingSynapse) -> dict:
        # iterate over the response headers and extract the necessary data
        headers = {
            k.decode("utf-8"): v.decode("utf-8")
            for k, v in response.__dict__["_raw_headers"]
        }
        # helper function to extract data from headers
        def extract_info(prefix):
            return {
                key.split("_")[-1]: value
                for key, value in headers.items()
                if key.startswith(prefix)
            }
        # return the extracted data in the expected format
        return {
            "name": headers.get("name", ""),
            "timeout": float(headers.get("timeout", 0)),
            "total_size": int(headers.get("total_size", 0)),
            "header_size": int(headers.get("header_size", 0)),
            "dendrite": extract_info("bt_header_dendrite"), # dendrite info
            "axon": extract_info("bt_header_axon"), # axon info
            "messages": self.messages, # field object
        }
```

[Here](https://github.com/opentensor/text-prompting/blob/main/prompting/protocol.py#L131) is a full example implementation of a streaming protocol based on the text-prompting network.

Please read the docstrings provided, they can be very helpful!

### Writing the server
Great! Now we have our protocol defined, let's see how to define our server.
This will generate the tokens to be streamed in this prompting example.

For brevity we will not be building a full miner, but inspecting the central components.
```python
class MyStreamPromptingMiner(bt.Miner):
    ... # any relevant methods you'd need for your miner

    # define your server forward here
    # NOTE: It is crucial that your typehints are correct and reflect your streaming protocol object
    # otherwise the axon will reject adding your route to the server.
    def forward(self, synapse: MyStreamingSynapse) -> MyStreamingSynapse:
        # Let's use a GPT2 tokenizer for this toy example
        tokenizer = GPT2Tokenizer.from_pretrained("gpt2")

        # Simulated function to decode token IDs into strings. In a real-world scenario,
        # this can be replaced with an actual model inference step.
        def model(ids):
            return (tokenizer.decode(id) for id in ids)

        # This function is called asynchronously to process the input text and send back tokens
        # as a streaming response. It essentially produces the async generator that will be
        # consumed by the client with an `async for` loop.
        async def _forward(text: str, send: Send):
            # `text` may be the input prompt to your model in a real-world scenario.
            # let's tokenize them into IDs for the sake of this example.
            input_ids = tokenizer(text, return_tensors="pt").input_ids.squeeze()
            
            # You may want to buffer your tokens before sending them back to the client.
            # this can be useful so we aren't flooding the client with individual tokens
            # and allows you more fine-grained control over how much data is sent back 
            # with each yield.
            N = 3  # Number of tokens to send back to the client at a time
            buffer = []
            # Iterate over the tokens and send the generationed tokens back to the client  
            # when we have sufficient (N) tokens in the buffer.       
            for token in model(input_ids):
                buffer.append(token) # Add token to buffer

                # If buffer has N tokens, send them back to the client.
                if len(buffer) == N:
                    joined_buffer = "".join(buffer)
                    # Send the tokens back to the client
                    # This is the core of the streaming response and the format 
                    # is important. The `send` function is provided by the ASGI server
                    # and is responsible for sending the response back to the client.
                    # This buffer will be received by the client as a single chunk of
                    # data, which can then be split into individual tokens!
                    await send(
                        {
                            "type": "http.response.body",
                            "body": joined_buffer.encode("utf-8"),
                            "more_body": True,
                        }
                    )
                    buffer = []  # Clear the buffer for next batch of tokens

        # Create a streaming response object using the `_forward` function
        # It is useful to wrap your _forward function in a partial function
        # to pass in the text argument lazily.
        token_streamer = partial(_forward, synapse.messages[0])
        # Return the streaming response object, which is an instance of the
        # `BTStreamingResponse` class.
        return synapse.create_streaming_response(token_streamer)
```

#### Complete Example
Here is a full example for reference:
> This inherits from the prompting (text-prompting) miner base class.
> Take a look at the `prompting/baseminer/miner.py` file [here](https://github.com/opentensor/text-prompting/blob/main/prompting/baseminer/miner.py) for more details.

```python
class StreamingTemplateMiner(prompting.Miner):
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
        bt.logging.trace("In outer PROMPT()")
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
            bt.logging.trace("In inner _PROMPT()")
            input_ids = tokenizer(text, return_tensors="pt").input_ids.squeeze()
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
```

### Writing the client
Excellent! Now we have defined our server, now we can define our client.

This has assumed you have:
1. Registered your miner on the chain (`finney`/`test`)
2. Are serving your miner on an open port (e.g. `12345`)

Steps:
- Instantiate your synapse subclass with the relevant information. E.g. `messages`, `roles`, etc.
- Instantiate your wallet and a dendrite client
- Query the dendrite client with your synapse object
- Iterate over the async generator to extract the yielded tokens on the server side

```python

# Import bittensor
import bittensor as bt

# Create your streaming synapse subclass object to house the request body
syn = MyStreamingSynapse(
    roles=["user"],
    messages=["hello this is a test of a streaming response. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."]
)

# Create a wallet instance that must be registered on the network
wallet = bt.wallet(name="default", hotkey="default")

# Instantiate the metagraph
metagraph = bt.metagraph(
    netuid=8, network="test", sync=True, lite=False
)

# Grab the axon you're serving
my_uid = 1
axon = metagraph.axons[my_uid]

# Create a Dendrite instance to handle client-side communication.
dendrite = bt.dendrite(wallet=wallet)


This is an async function so we can use the `await` keyword when querying the server with the dendrite object.
async def main():
    # Send a request to the Axon using the Dendrite, passing in a StreamPrompting 
    # instance with roles and messages. The response is awaited, as the Dendrite 
    # communicates asynchronously with the Axon. Returns a list of async generator.
    responses = await dendrite(
        [axon],
        syn,
        deserialize=False,
        streaming=True
    )

    # Now that we have our responses we want to iterate over the yielded tokens
    # iterate over the async generator to extract the yielded tokens on server side
    for resp in responses:
        i=0
        async for chunk in resp:
            i += 1
            if i % 5 == 0:
                print()
            if isinstance(chunk, list):
                print(chunk[0], end="", flush=True)
            else:
                # last object yielded is the synapse itself with completion filled
                synapse = chunk
        break

    # The synapse object contains the completion attribute which contains the
    # accumulated tokens from the streaming response.

if __name__ == "__main__":
    # Run the main function with asyncio
    asyncio.run(main())
    
```
There you have it!

### Complete example
If you would like to see a complete standalone example that only depends on bittensor>=6.2.0, look below:

- client.py
- streaming_miner.py
- 

# client.py
```python
# Import bittensor and the text-prompting packages
import bittensor as bt
import prompting

# Create a StreamPrompting synapse object to house the request body
syn = prompting.protocol.StreamPrompting(
    roles=["user"], 
    messages=["hello this is a test of a streaming response. Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."])
syn

# create a wallet instance that must be registered on the network
wallet = bt.wallet(name="default", hotkey="default")
wallet

# instantiate the metagraph
metagraph = bt.metagraph(
    netuid=8, network="test", sync=True, lite=False
)
metagraph

# Grab the axon you're serving
axon = metagraph.axons[62]
axon

# Create a Dendrite instance to handle client-side communication.
d = bt.dendrite(wallet=wallet)
d


async def main():
        
    # Send a request to the Axon using the Dendrite, passing in a StreamPrompting 
    # instance with roles and messages. The response is awaited, as the Dendrite 
    # communicates asynchronously with the Axon. Returns a list of async generator.
    responses = await d(
        [axon],
        syn,
        deserialize=False,
        streaming=True
    )
    responses 

    # iterate over the async generator to extract the yielded tokens on server side
    for resp in responses:
        i=0
        async for chunk in resp:
            i += 1
            if i % 5 == 0:
                print()
            if isinstance(chunk, list):
                print(chunk[0], end="", flush=True)
            else:
                # last object yielded is the synapse itself with completion filled
                synapse = chunk
        break

if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
```
