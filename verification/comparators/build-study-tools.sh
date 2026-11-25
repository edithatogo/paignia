#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo"
mkdir "$1"
build="$(cd "$1" && pwd)"
python="$repo/target/dev-python/bin/python"
provider=a04a19da06cc50e310a172574ed66f62000b891a
export CC="$(command -v cc)"
export CXX="$(command -v c++)"

{
  uname -ms
  case "$(uname -s)" in
    Darwin)
      sysctl machdep.cpu.brand_string hw.logicalcpu
      /usr/bin/time -l /usr/bin/true
      ;;
    Linux)
      lscpu
      /usr/bin/time --version
      ;;
    *)
      exit 1
      ;;
  esac
  "$python" --version
  rustup run 1.75.0 rustc --version --verbose
  "$CC" --version
  "$CXX" --version
  cmake --version
  make --version
  ls -l /usr/bin/time
} > "$build/environment.txt" 2>&1

shasum -a 256 verification/comparators/build-study-tools.sh | cut -d ' ' -f 1 > "$build/recipe.sha256"

(
  set -x
  cd "$build"
  curl --fail --silent --show-error --location --retry 2 --output gambit.tar.gz \
    https://github.com/gambitproject/gambit/releases/download/v16.7.0/gambit-16.7.0.tar.gz
  curl --fail --silent --show-error --location --retry 2 --output open-spiel.tar.gz \
    https://files.pythonhosted.org/packages/bb/3b/9d56d511eff27e26714aa232c945bb00335f792bc1959637ee0d2639a018/open_spiel-2.0.2.tar.gz
  shasum -a 256 --check <<'SHA256'
35a2d7df4f8181cb216ce9a2b10ca820724768d8c69787a608a4a2725a745278  gambit.tar.gz
6f520dfe499b9e5e1e3a31cf77bf071abb02b98104789e7698021922a064e856  open-spiel.tar.gz
SHA256
  tar -xzf gambit.tar.gz
  tar -xzf open-spiel.tar.gz
  (
    cd gambit-16.7.0
    if [ "$(uname -s)" = Darwin ]; then
      gmp="$(brew --prefix gmp)"
      export CPPFLAGS="-I$gmp/include" LDFLAGS="-L$gmp/lib"
    fi
    ./configure --disable-gui
    make -j4 GAMBIT_VERSION=16.7.0 gambit-enumpure gambit-enummixed
  )
  cmake -S open_spiel-2.0.2/open_spiel -B open-spiel-build \
    -DCMAKE_BUILD_TYPE=Release -DPython3_EXECUTABLE="$python" \
    -DCMAKE_LIBRARY_OUTPUT_DIRECTORY="$build/open-spiel-library" \
    -DOPEN_SPIEL_BUILD_WITH_PYTHON=ON -DOPEN_SPIEL_BUILD_WITH_ACPC=OFF \
    -DOPEN_SPIEL_BUILD_WITH_HANABI=OFF -DOPEN_SPIEL_BUILD_WITH_GAMUT=OFF \
    -DOPEN_SPIEL_BUILD_WITH_LIBNOP=OFF -DOPEN_SPIEL_BUILD_WITH_LIBTORCH=OFF \
    -DOPEN_SPIEL_BUILD_WITH_ORTOOLS=OFF -DOPEN_SPIEL_BUILD_WITH_ROSHAMBO=OFF \
    -DOPEN_SPIEL_BUILD_WITH_XINXIN=OFF
  cmake --build open-spiel-build --target pyspiel --parallel 4

  test "$(git -C "$repo" rev-parse 'refs/tags/v0.1.0^{}')" = "$provider"
  git -C "$repo" worktree add --detach "$build/provider" "$provider"
  CARGO_TARGET_DIR="$build/paignia-build" rustup run 1.75.0 cargo build --release --locked \
    --manifest-path "$build/provider/crates/paignia-io/Cargo.toml" --examples
) > "$build/build.log" 2>&1

(
  cd "$build"
  shasum -a 256 environment.txt recipe.sha256 build.log gambit.tar.gz open-spiel.tar.gz provider/Cargo.lock \
    gambit-16.7.0/gambit-enumpure gambit-16.7.0/gambit-enummixed open-spiel-library/pyspiel.so \
    paignia-build/release/examples/{solve,solve_profile} > build.sha256
)
