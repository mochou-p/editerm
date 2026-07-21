{
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "editerm";
  version = "git-main";

  buildType = "release";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    # removes the need for outputHashes
    allowBuiltinFetchGit = true;
  };

  meta = {
    mainProgram = "editerm";
    description = "a based text-based text editor";
    homepage = "https://github.com/mochou-p/editerm";
    license = with lib.licenses; [ asl20 mit ];
    maintainers = [ "mochou-p" "cggjaicf" ];
  };
})
