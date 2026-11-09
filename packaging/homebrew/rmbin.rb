class Rmbin < Formula
  desc "Offline file and folder remover. Overwrites with a plain text pattern, syncs to disk, then deletes. No network access, no telemetry. States its limits."
  homepage "https://pesty.sh"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/pesty-sh/rmbin.git", branch: "main"

  # After tag v0.0.1 is on GitHub, uncomment these and paste the sha256 of
  # GitHub's tarball, not of a local git archive.
  # url "https://github.com/pesty-sh/rmbin/archive/refs/tags/v0.0.1.tar.gz"
  # sha256 "PASTE_SHA256_OF_GITHUB_TARBALL"

  depends_on "rust" => :build

  def fetch
    system "cargo", "fetch", *std_cargo_fetch_args
  end

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "rmbin", shell_output("#{bin}/rmbin --version")
    assert_match "best-effort", shell_output("#{bin}/rmbin --help")
  end
end
