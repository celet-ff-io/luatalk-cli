rem This batch file is an easy usage of `luatalk.exe do <FILE> typst-compile -f <FORMAT>`,
rem where you could drag your Lua file as <FILE>.
rem Get `luatalk.exe`: From `https://github.com/celet-ff-io/luatalk-cli
rem   or `cargo install luatalk-cli`
rem Get `typst.exe`: From `https://github.com/typst/typst`
rem   or `cargo install typst-cli`

@echo off
chcp 65001

if "%~1"=="" (
    echo No file specified.
    pause
    exit
)

rem Edit your variables below

set "LUATALK=luatalk.exe"
set "LUATALK__DO_TYPST_COMPILE__TYPST_COMMAND=typst.exe"

set "FONT_FAMILY=Microsoft YaHei"
set "WIDTH=1080"
set "FONT_SIZE=20"
set "LENGTH_FACTOR=1.0"

set "FORMAT=pdf"
"luatalk.exe" do "%~1" typst-compile -f "%FORMAT%" --font-family "%FONT_FAMILY%" --width "%WIDTH%" --font-size "%FONT_SIZE%" --length-factor "%LENGTH_FACTOR%"
pause
