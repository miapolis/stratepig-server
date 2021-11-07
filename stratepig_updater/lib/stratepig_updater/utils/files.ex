defmodule StratepigUpdater.Utils.Files do
  def priv_dir() do
    "/files/priv"
  end

  def static_asset(path) do
    Path.join(priv_dir(), "/static/#{path}")
  end

  def update_file(:launcher) do
    static_asset("updates/launcher.txt")
  end

  def update_file(:game) do
    static_asset("updates/game.txt")
  end

  def version_file(:launcher) do
    static_asset("versions/launcher.txt")
  end

  def version_file(:game) do
    static_asset("versions/game.txt")
  end

  @spec binary(:launcher, any()) :: any()
  def binary(:launcher, platform) do
    static_asset("bin/#{platform}/Launcher.zip")
  end

  @spec binary(:game, any()) :: any()
  def binary(:game, platform) do
    static_asset("bin/#{platform}/Build.zip")
  end
end
