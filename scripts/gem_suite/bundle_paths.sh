# Where a gem's suite finds the gems its tests load, under GEM_BUNDLES. Every
# suite shares one directory but simplecov's, whose sandbox specs install
# projects of their own, Rails among them, with gems whose C extensions
# Metorex does not build. Bundler warns of each of those in every suite whose
# directory holds it.
GEM_BUNDLES="${GEM_BUNDLES:-${CARGO_TARGET_DIR:-$REPO_ROOT/target}/gem_bundles}"

bundle_path_of() {
  case "$1" in
    simplecov) echo "$GEM_BUNDLES/simplecov" ;;
    *) echo "$GEM_BUNDLES/shared" ;;
  esac
}
