{-# LANGUAGE MultiWayIf    #-}
{-# LANGUAGE UnicodeSyntax #-}

import Hake

import Data.List (intercalate)

main ∷ IO ()
main = hake $ do

  "clean | clean the project" ∫
    cargo ["clean"] >> removeDirIfExists targetPath

  "update | update dependencies" ∫ cargo ["update"]

  salieriExecutable ♯
    cargo <| "build" : buildFlagsSalieri

  amadeusExecutable ♯
    cargo <| "build" : buildFlagsAmadeus

  "install | install to system" ◉ [amadeusExecutable] ∰
    cargo <| "install" : buildFlagsAmadeus

  "test | build and test" ◉ [amadeusExecutable] ∰ do
    cargo ["test"]
    cargo ["clippy"]
    rawSystem amadeusExecutable ["--version"]
      >>= checkExitCode

 where
  appNameSalieri ∷ String
  appNameSalieri = "salieri"

  appNameAmadeus ∷ String
  appNameAmadeus = "amadeus"

  targetPath ∷ FilePath
  targetPath = "target"

  buildPath ∷ FilePath
  buildPath = targetPath </> "release"

  features ∷ [String]
  features = [ "trackers"
             , "torch" ]

  buildFlagsSalieri ∷ [String]
  buildFlagsSalieri = [ "-p", appNameSalieri, "--release" ]

  buildFlagsAmadeus ∷ [String]
  buildFlagsAmadeus = [ "-p", appNameAmadeus
                      , "--release", "--features"
                      , intercalate "," features ]

  salieriExecutable ∷ FilePath
  salieriExecutable =
    {- HLINT ignore "Redundant multi-way if" -}
    if | os ∈ ["win32", "mingw32", "cygwin32"] → buildPath </> appNameSalieri ++ ".exe"
       | otherwise → buildPath </> appNameSalieri

  amadeusExecutable ∷ FilePath
  amadeusExecutable =
    {- HLINT ignore "Redundant multi-way if" -}
    if | os ∈ ["win32", "mingw32", "cygwin32"] → buildPath </> appNameAmadeus ++ ".exe"
       | otherwise → buildPath </> appNameAmadeus
