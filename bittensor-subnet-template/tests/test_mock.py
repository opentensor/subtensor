import pytest
import asyncio
import bittensor as bt
from prompting.mock import MockDendrite, MockMetagraph, MockSubtensor
from prompting.protocol import PromptingSynapse


@pytest.mark.parametrize("netuid", [1, 2, 3])
@pytest.mark.parametrize("n", [2, 4, 8, 16, 32, 64])
@pytest.mark.parametrize("wallet", [bt.MockWallet(), None])
def test_mock_subtensor(netuid, n, wallet):
    subtensor = MockSubtensor(netuid=netuid, n=n, wallet=wallet)
    neurons = subtensor.neurons(netuid=netuid)
    # Check netuid
    assert subtensor.subnet_exists(netuid)
    # Check network
    assert subtensor.network == "mock"
    assert subtensor.chain_endpoint == "mock_endpoint"
    # Check number of neurons
    assert len(neurons) == (n + 1 if wallet is not None else n)
    # Check wallet
    if wallet is not None:
        assert subtensor.is_hotkey_registered(
            netuid=netuid, hotkey_ss58=wallet.hotkey.ss58_address
        )

    for neuron in neurons:
        assert type(neuron) == bt.NeuronInfo
        assert subtensor.is_hotkey_registered(
            netuid=netuid, hotkey_ss58=neuron.hotkey
        )


@pytest.mark.parametrize("n", [16, 32, 64])
def test_mock_metagraph(n):
    mock_subtensor = MockSubtensor(netuid=1, n=n)
    mock_metagraph = MockMetagraph(subtensor=mock_subtensor)
    # Check axons
    axons = mock_metagraph.axons
    assert len(axons) == n
    # Check ip and port
    for axon in axons:
        assert type(axon) == bt.AxonInfo
        assert axon.ip == mock_metagraph.default_ip
        assert axon.port == mock_metagraph.default_port


def test_mock_reward_pipeline():
    pass


def test_mock_neuron():
    pass


@pytest.mark.parametrize("timeout", [0.1, 0.2])
@pytest.mark.parametrize("min_time", [0, 0.05, 0.1])
@pytest.mark.parametrize("max_time", [0.1, 0.15, 0.2])
@pytest.mark.parametrize("n", [4, 16, 64])
def test_mock_dendrite_timings(timeout, min_time, max_time, n):
    mock_wallet = None
    mock_dendrite = MockDendrite(mock_wallet)
    mock_dendrite.min_time = min_time
    mock_dendrite.max_time = max_time
    mock_subtensor = MockSubtensor(netuid=1, n=n)
    mock_metagraph = MockMetagraph(subtensor=mock_subtensor)
    axons = mock_metagraph.axons

    async def run():
        return await mock_dendrite(
            axons,
            synapse=PromptingSynapse(
                roles=["user"], messages=["What is the capital of France?"]
            ),
            timeout=timeout,
        )

    responses = asyncio.run(run())
    for synapse in responses:
        assert (
            hasattr(synapse, "dendrite")
            and type(synapse.dendrite) == bt.TerminalInfo
        )

        dendrite = synapse.dendrite
        # check synapse.dendrite has (process_time, status_code, status_message)
        for field in ("process_time", "status_code", "status_message"):
            assert (
                hasattr(dendrite, field)
                and getattr(dendrite, field) is not None
            )

        # check that the dendrite take between min_time and max_time
        assert min_time <= dendrite.process_time
        assert dendrite.process_time <= max_time + 0.1
        # check that responses which take longer than timeout have 408 status code
        if dendrite.process_time >= timeout + 0.1:
            assert dendrite.status_code == 408
            assert dendrite.status_message == "Timeout"
            assert synapse.dummy_output == synapse.dummy_input
        # check that responses which take less than timeout have 200 status code
        elif dendrite.process_time < timeout:
            assert dendrite.status_code == 200
            assert dendrite.status_message == "OK"
            # check that outputs are not empty for successful responses
            assert synapse.dummy_output == synapse.dummy_input * 2
        # dont check for responses which take between timeout and max_time because they are not guaranteed to have a status code of 200 or 408
