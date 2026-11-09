class Rmbin < Formula
  desc "Offline file and folder remover. Overwrites with a plain text pattern, syncs to disk, then deletes. No network access, no telemetry. States its limits."
  homepage "https://pesty.sh"
  license any_of: ["MIT", "Apache-2.0"]

  # Stable install: one universal CLI binary (Intel + Apple silicon).
  # Not an .app, and not one build per macOS version.
  # After the v0.0.1 GitHub Release exists, uncomment these two lines.
  # sha256 is the line for this file in SHA256SUMS, not a local hash.
  # url "https://github.com/pesty-sh/rmbin/releases/download/v0.0.1/rmbin-universal-apple-darwin.tar.gz"
  # sha256 "PASTE_SHA256_FROM_SHA256SUMS"

  # Latest commit on main. This path compiles with cargo.
  head "https://github.com/pesty-sh/rmbin.git", branch: "main"

  # Rust is only needed for --HEAD. A stable install unpacks the universal binary.
  depends_on "rust" => :build if build.head?

  def install
    if build.head?
      system "cargo", "install", *std_cargo_args
    else
      bin.install "rmbin"
    end
  end

  test do
    assert_match "rmbin", shell_output("#{bin}/rmbin --version")
    assert_match "best-effort", shell_output("#{bin}/rmbin --help")
    assert_match "No warranty", shell_output("#{bin}/rmbin --help")
  end
end
