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

  vivaldiExecutable ♯
    cargo <| "build" : buildFlagsVivaldi False

  amadeusExecutable ◉ [salieriExecutable] ♯♯
    cargo <| "build" : buildFlagsAmadeus False

  "fat | build Amadeus and Salieri with fat LTO" ∫
       cargo <| "build" : buildFlagsSalieri True
    >> cargo <| "build" : buildFlagsVivaldi True
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
    systemctl ["restart", appNameVivaldi]
    systemctl ["restart", appNameAmadeus]

  "run | run Amadeus" ◉ [ amadeusExecutable ] ∰ do
    cargo . (("run" : buildFlagsAmadeus False) ++) . ("--" :) =<< getHakeArgs

 where
  -- to use system PyTorch instead of downloaded
  {-
  setTorchEnv ∷ IO ()
  setTorchEnv = setEnv "LIBTORCH_USE_PYTORCH" "1"
             >> setEnv "LIBTORCH_BYPASS_VERSION_CHECK" "1"
  -}

  appNameSalieri ∷ String
  appNameSalieri = "salieri"

  appNameVivaldi ∷ String
  appNameVivaldi = "vivaldi"

  appNameAmadeus ∷ String
  appNameAmadeus = "amadeus"

  targetPath ∷ FilePath
  targetPath = "target"

  buildPath ∷ FilePath
  buildPath = targetPath </> "release"

  amadeusFeatures ∷ [String]
  amadeusFeatures = [ "trackers"
                    , "torch" ]

  fatArgs ∷ [String]
  fatArgs = [ "--profile"
            , "fat-release" ]

  buildFlagsSalieri ∷ Bool -> [String]
  buildFlagsSalieri fat =
    let defaultFlags = [ "-p", appNameSalieri
                       , "--release" ]
    in if fat then defaultFlags ++ fatArgs
              else defaultFlags

  buildFlagsVivaldi ∷ Bool -> [String]
  buildFlagsVivaldi fat =
    let defaultFlags = [ "-p", appNameVivaldi
                       , "--release" ]
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
  salieriExecutable = buildPath </> appNameSalieri

  vivaldiExecutable ∷ FilePath
  vivaldiExecutable = buildPath </> appNameVivaldi

  amadeusExecutable ∷ FilePath
  amadeusExecutable = buildPath </> appNameAmadeus
