{-# LANGUAGE MultiWayIf    #-}
{-# LANGUAGE UnicodeSyntax #-}

import Hake

import Data.List (intercalate)

main ∷ IO ()
main = hake $ do

  "clean | clean the project" ∫
    cargo ["clean"] >> removeDirIfExists targetPath

  "update | update dependencies" ∫ cargo ["update"]

  amadeusExecutable ♯
    cargo <| "build" : buildFlags

  "install | install to system" ◉ [amadeusExecutable] ∰
    cargo <| "install" : buildFlags

  "test | build and test" ◉ [amadeusExecutable] ∰ do
    cargo ["test"]
    cargo ["clippy"]
    rawSystem amadeusExecutable ["--version"]
      >>= checkExitCode

 where
  appName ∷ String
  appName = "amadeus"

  targetPath ∷ FilePath
  targetPath = "target"

  buildPath ∷ FilePath
  buildPath = targetPath </> "release"

  features ∷ [String]
  features = [ "trackers"
             , "torch" ]

  buildFlags ∷ [String]
  buildFlags = [ "--release", "--features"
               , intercalate "," features ]

  amadeusExecutable ∷ FilePath
  amadeusExecutable =
    {- HLINT ignore "Redundant multi-way if" -}
    if | os ∈ ["win32", "mingw32", "cygwin32"] → buildPath </> appName ++ ".exe"
       | otherwise → buildPath </> appName
