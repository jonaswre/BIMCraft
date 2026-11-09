param([Parameter(Mandatory=$true)][string]$Cli)
$ErrorActionPreference = 'Stop'
$Cli = (Resolve-Path -LiteralPath $Cli).Path
Push-Location $PSScriptRoot
try {
    & $Cli run --script journey.txt --cmd 'render {"path":"after-centre.png","width":1000,"height":800}' --save after.bimcraft --save after.ifc > after-cli.jsonl
    if ($LASTEXITCODE -ne 0) { throw 'CLI creation failed' }
    & $Cli run after.bimcraft --cmd 'elements {"category":"room"}' --cmd 'elements {"category":"roomseparator"}' --cmd 'room_area {"at":"finish"}' --cmd 'elements {"category":"room"}' --cmd 'render {"path":"after-finish.png","width":1000,"height":800}' --save after-finish.ifc > after-native-reopen.jsonl
    if ($LASTEXITCODE -ne 0) { throw 'Native reopen failed' }
    & $Cli run after.ifc --cmd 'room_area {"at":"centre"}' --cmd 'elements {"category":"room"}' --cmd 'room_area {"at":"finish"}' --cmd 'elements {"category":"room"}' > after-ifc-reopen.jsonl
    if ($LASTEXITCODE -ne 0) { throw 'IFC reopen failed' }
    & $Cli run before.bimcraft --cmd 'elements {"category":"room"}' > after-open-before-native.jsonl
    if ($LASTEXITCODE -ne 0) { throw 'Original native reopen failed' }
} finally {
    Pop-Location
}
