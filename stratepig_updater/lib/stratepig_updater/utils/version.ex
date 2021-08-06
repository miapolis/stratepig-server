defmodule StratepigUpdater.Utils.Version do
  alias StratepigUpdater.Utils.Files

  def init(_opts) do
    :ets.new(:version_info, [:named_table])

    {:ok, launcher_version} = File.read(Files.version_file(:launcher))
    {:ok, game_version} = File.read(Files.version_file(:game))
    :ets.insert(:version_info, {"launcher", launcher_version})
    :ets.insert(:version_info, {"game", game_version})

    IO.puts("Initialized version files {#{launcher_version}, #{game_version}}")
  end

  @spec project_version() :: String.t()
  def project_version() do
    [{"game", contents}] = :ets.lookup(:version_info, "game")
    contents
  end

  @spec launcher_version() :: String.t()
  def launcher_version() do
    [{"launcher", contents}] = :ets.lookup(:version_info, "launcher")
    contents
  end
end
