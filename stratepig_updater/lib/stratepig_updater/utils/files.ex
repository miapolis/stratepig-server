defmodule StratepigUpdater.Utils.Files do
  @type updatable :: :launcher | :game

  def priv_dir() do
    :code.priv_dir(:stratepig_updater)
  end

  def static_asset(path) do
    Path.join(priv_dir(), "/static/#{path}")
  end

  @spec update_file(updatable) :: any()
  def update_file(typ) do
    case typ do
      :launcher -> static_asset("updates/launcher.txt")
      :game -> static_asset("updates/game.txt")
    end
  end

  @spec binary(updatable) :: any()
  def binary(typ) do
    case typ do
      :launcher -> static_asset("bin/Launcher.exe")
      :game -> static_asset("bin/Build.zip")
    end
  end
end
