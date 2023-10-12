defmodule Pixelflut.Net.TcpClientPool do
  require Logger
  use GenServer
  @behaviour NimblePool

  def test_start_link(host \\ "localhost", port \\ 1234) do
    NimblePool.start_link(worker: {__MODULE__, {host, port}})
  end

  def test_stop_link(pool) do
    NimblePool.stop(pool)
  end

  @impl NimblePool
  def init_worker({host, port} = pool_state) do
    # connect socket to server
    Logger.debug("Connecting worker to tcp://#{host}:#{port}")
    {:ok, sock} = :gen_tcp.connect(String.to_charlist(host), port, [])

    # return pool state with connected socket
    {:ok, sock, pool_state}
  end

  @impl NimblePool
  def terminate_worker(_reason, sock, pool_state) do
    Logger.debug("Disconnecting worker from server")
    :ok = :gen_tcp.close(sock)
    {:ok, pool_state}
  end

  @impl NimblePool
  def handle_checkout(_maybe_wrapped_command, _from, worker_state, pool_state) do
    {:ok, nil, worker_state, pool_state}
  end
end
