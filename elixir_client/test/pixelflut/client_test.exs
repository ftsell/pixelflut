defmodule Pixelflut.ClientTest do
  use ExUnit.Case
  doctest Pixelflut.Client

  test "greets the world" do
    assert Pixelflut.Client.hello() == :world
  end
end
