{-# LANGUAGE MultiWayIf    #-}
{-# LANGUAGE UnicodeSyntax #-}

import Hake

main ∷ IO ()
main = hake $ do
  "clean | clean the project" ∫
    cargo ["clean"] >> removeDirIfExists "target"

  amadeusExecutable ♯ do
    cargo <| "build" : buildFlags

  "install | install to system" ◉ [amadeusExecutable] ∰
    cargo <| "install" : buildFlags

  "test | build and test" ◉ [amadeusExecutable] ∰ do
    cargo ["test"]
    rawSystem amadeusExecutable ["--version"]
      >>= checkExitCode

 where buildPath ∷ String
       buildPath = "target/release"

       features ∷ String
       features = "trackers,torch"

       buildFlags ∷ [String]
       buildFlags = ["--release", "--features", features]

       amadeusExecutable ∷ String
       amadeusExecutable =
         {- HLINT ignore "Redundant multi-way if" -}
         if | os ∈ ["win32", "mingw32", "cygwin32"] → buildPath </> "amadeus.exe"
            | otherwise → buildPath </> "amadeus"
