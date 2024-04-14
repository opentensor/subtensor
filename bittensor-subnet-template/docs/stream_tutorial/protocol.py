import pydantic
import bittensor as bt

from abc import ABC, abstractmethod
from typing import List, Union, Callable, Awaitable
from starlette.responses import StreamingResponse


class StreamPrompting(bt.StreamingSynapse):
    """
    StreamPrompting is a specialized implementation of the `StreamingSynapse` tailored for prompting functionalities within
    the Bittensor network. This class is intended to interact with a streaming response that contains a sequence of tokens,
    which represent prompts or messages in a certain scenario.

    As a developer, when using or extending the `StreamPrompting` class, you should be primarily focused on the structure
    and behavior of the prompts you are working with. The class has been designed to seamlessly handle the streaming,
    decoding, and accumulation of tokens that represent these prompts.

    Attributes:
    - `roles` (List[str]): A list of roles involved in the prompting scenario. This could represent different entities
                           or agents involved in the conversation or use-case. They are immutable to ensure consistent
                           interaction throughout the lifetime of the object.

    - `messages` (List[str]): These represent the actual prompts or messages in the prompting scenario. They are also
                              immutable to ensure consistent behavior during processing.

    - `completion` (str): Stores the processed result of the streaming tokens. As tokens are streamed, decoded, and
                          processed, they are accumulated in the completion attribute. This represents the "final"
                          product or result of the streaming process.
    - `required_hash_fields` (List[str]): A list of fields that are required for the hash.

    Methods:
    - `process_streaming_response`: This method asynchronously processes the incoming streaming response by decoding
                                    the tokens and accumulating them in the `completion` attribute.

    - `deserialize`: Converts the `completion` attribute into its desired data format, in this case, a string.

    - `extract_response_json`: Extracts relevant JSON data from the response, useful for gaining insights on the response's
                               metadata or for debugging purposes.

    Note: While you can directly use the `StreamPrompting` class, it's designed to be extensible. Thus, you can create
    subclasses to further customize behavior for specific prompting scenarios or requirements.
    """

    roles: List[str] = pydantic.Field(
        ...,
        title="Roles",
        description="A list of roles in the StreamPrompting scenario. Immuatable.",
        allow_mutation=False,
    )

    messages: List[str] = pydantic.Field(
        ...,
        title="Messages",
        description="A list of messages in the StreamPrompting scenario. Immutable.",
        allow_mutation=False,
    )

    required_hash_fields: List[str] = pydantic.Field(
        ["messages"],
        title="Required Hash Fields",
        description="A list of required fields for the hash.",
        allow_mutation=False,
    )

    completion: str = pydantic.Field(
        "",
        title="Completion",
        description="Completion status of the current StreamPrompting object. This attribute is mutable and can be updated.",
    )

    async def process_streaming_response(self, response: StreamingResponse):
        """
        `process_streaming_response` is an asynchronous method designed to process the incoming streaming response from the
        Bittensor network. It's the heart of the StreamPrompting class, ensuring that streaming tokens, which represent
        prompts or messages, are decoded and appropriately managed.

        As the streaming response is consumed, the tokens are decoded from their 'utf-8' encoded format, split based on
        newline characters, and concatenated into the `completion` attribute. This accumulation of decoded tokens in the
        `completion` attribute allows for a continuous and coherent accumulation of the streaming content.

        Args:
            response: The streaming response object containing the content chunks to be processed. Each chunk in this
                      response is expected to be a set of tokens that can be decoded and split into individual messages or prompts.
        """
        if self.completion is None:
            self.completion = ""
        bt.logging.debug(
            "Processing streaming response (StreamingSynapse base class)."
        )
        async for chunk in response.content.iter_any():
            bt.logging.debug(f"Processing chunk: {chunk}")
            tokens = chunk.decode("utf-8").split("\n")
            for token in tokens:
                bt.logging.debug(f"--processing token: {token}")
                if token:
                    self.completion += token
            bt.logging.debug(f"yielding tokens {tokens}")
            yield tokens

    def deserialize(self) -> str:
        """
        Deserializes the response by returning the completion attribute.

        Returns:
            str: The completion result.
        """
        return self.completion

    def extract_response_json(self, response: StreamingResponse) -> dict:
        """
        `extract_response_json` is a method that performs the crucial task of extracting pertinent JSON data from the given
        response. The method is especially useful when you need a detailed insight into the streaming response's metadata
        or when debugging response-related issues.

        Beyond just extracting the JSON data, the method also processes and structures the data for easier consumption
        and understanding. For instance, it extracts specific headers related to dendrite and axon, offering insights
        about the Bittensor network's internal processes. The method ultimately returns a dictionary with a structured
        view of the extracted data.

        Args:
            response: The response object from which to extract the JSON data. This object typically includes headers and
                      content which can be used to glean insights about the response.

        Returns:
            dict: A structured dictionary containing:
                - Basic response metadata such as name, timeout, total_size, and header_size.
                - Dendrite and Axon related information extracted from headers.
                - Roles and Messages pertaining to the current StreamPrompting instance.
                - The accumulated completion.
        """
        headers = {
            k.decode("utf-8"): v.decode("utf-8")
            for k, v in response.__dict__["_raw_headers"]
        }

        def extract_info(prefix):
            return {
                key.split("_")[-1]: value
                for key, value in headers.items()
                if key.startswith(prefix)
            }

        return {
            "name": headers.get("name", ""),
            "timeout": float(headers.get("timeout", 0)),
            "total_size": int(headers.get("total_size", 0)),
            "header_size": int(headers.get("header_size", 0)),
            "dendrite": extract_info("bt_header_dendrite"),
            "axon": extract_info("bt_header_axon"),
            "roles": self.roles,
            "messages": self.messages,
            "completion": self.completion,
        }
