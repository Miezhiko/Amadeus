echo off

$Env:LIBTORCH = "C:\ProgramData\Anaconda3\envs\opencv\Lib\site-packages\torch\lib"
$Env:Path += ";C:\ProgramData\Anaconda3\envs\opencv\Lib\site-packages\torch\lib"

cargo build
