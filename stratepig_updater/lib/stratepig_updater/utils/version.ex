defmodule StratepigUpdater.Utils.Version do
  @spec project_version() :: String.t()
  def project_version() do
    {:ok, vsn} = :application.get_key(:stratepig_updater, :vsn)
    List.to_string(vsn)
  end

  @spec launcher_version() :: String.t()
  def launcher_version() do
    # Eh
    "0.2.2"
  end
end
