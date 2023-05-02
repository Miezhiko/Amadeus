{-# LANGUAGE MultiWayIf    #-}
{-# LANGUAGE UnicodeSyntax #-}

import           Hake

main ∷ IO ()
main = hake $ do

  "clean | clean the project" ∫
    cargo ["clean"] ?> removeDirIfExists targetPath

  "update | update dependencies" ∫ cargo ["update"]

  salieriExecutable ♯
    cargo <| "build" : buildFlagsSalieri False

  amadeusExecutable ◉ [salieriExecutable] ♯♯
    cargo <| "build" : buildFlagsAmadeus False

  "fat | build Amadeus and Salieri with fat LTO" ∫
       cargo <| "build" : buildFlagsSalieri True
    >> cargo <| "build" : buildFlagsAmadeus True

  "install | install to system" ◉ [ "fat" ] ∰
    cargo <| "install" : buildFlagsAmadeus True

  "test | build and test" ◉ [amadeusExecutable] ∰ do
    cargo ["test"]
    cargo ["clippy"]
    rawSystem amadeusExecutable ["--version"]
      >>= checkExitCode

  "restart | restart services" ◉ [ salieriExecutable
                                 , amadeusExecutable ] ∰ do
    systemctl ["restart", appNameSalieri]
    systemctl ["restart", appNameAmadeus]

  "run | run Amadeus" ◉ [ amadeusExecutable ] ∰ do
    cargo . (("run" : buildFlagsAmadeus False) ++) . ("--" :) =<< getHakeArgs

 where
  appNameSalieri ∷ String
  appNameSalieri = "salieri"

  appNameAmadeus ∷ String
  appNameAmadeus = "amadeus"

  targetPath ∷ FilePath
  targetPath = "target"

  buildPath ∷ FilePath
  buildPath = targetPath </> "release"

  salieriFeatures ∷ [String]
  salieriFeatures = [ "kafka" ]

  amadeusFeatures ∷ [String]
  amadeusFeatures = [ "trackers"
                    , "torch" ]

  fatArgs ∷ [String]
  fatArgs = [ "--profile"
            , "fat-release" ]

  buildFlagsSalieri ∷ Bool -> [String]
  buildFlagsSalieri fat =
    let defaultFlags = [ "-p", appNameSalieri
                       , "--release", "--features"
                       , intercalate "," salieriFeatures ]
    in if fat then defaultFlags ++ fatArgs
              else defaultFlags

  buildFlagsAmadeus ∷ Bool -> [String]
  buildFlagsAmadeus fat =
    let defaultFlags = [ "-p", appNameAmadeus
                       , "--release", "--features"
                       , intercalate "," amadeusFeatures ]
    in if fat then defaultFlags ++ fatArgs
              else defaultFlags

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
