param([Parameter(Mandatory=$true)][string]$Repo,[Parameter(Mandatory=$true)][string]$Target,[string]$Output=$PSScriptRoot)
$ErrorActionPreference='Stop'
$repoPath=[IO.Path]::GetFullPath($Repo)
$testRoot=Join-Path $repoPath 'crates/ui-egui/src/tests.rs'
$tempModule=Join-Path $repoPath 'crates/ui-egui/src/tests/task10_capture.rs'
if (Test-Path -LiteralPath $tempModule) { throw 'Temporary probe already exists; refusing to overwrite it.' }
$original=[IO.File]::ReadAllBytes($testRoot)
$captureOnlyMocks=@{}
foreach($relative in @('crates/ui-egui/src/tests/light_canvas.rs','crates/ui-egui/src/tests/background_grid.rs')) {
    $path=Join-Path $repoPath $relative
    $captureOnlyMocks[$path]=[IO.File]::ReadAllBytes($path)
}
$exitCode=1
try {
    # Existing upload-only mocks omit the WGPU-gated render method. This capture
    # never instantiates them; add an explicit error only while compiling the probe.
    foreach($path in $captureOnlyMocks.Keys) {
        $source=[IO.File]::ReadAllText($path)
        $source=$source -replace '(impl egui_kittest::TestRenderer for \w+ \{)', ('$1'+"`n"+'    fn render(&mut self, _: &egui::Context, _: &egui::FullOutput) -> Result<image::RgbaImage, String> { Err("Upload-only mock is not a capture renderer".into()) }'+"`n")
        [IO.File]::WriteAllText($path,$source)
    }
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'capture_industrial.rs') -Destination $tempModule
    [IO.File]::AppendAllText($testRoot,"`nmod task10_capture;`n")
    $env:CARGO_TARGET_DIR=$Target
    $env:CARGO_BUILD_JOBS='2'
    $env:RAYON_NUM_THREADS='2'
    $env:TASK10_UI_OUTPUT_DIR=[IO.Path]::GetFullPath($Output)
    Push-Location -LiteralPath $repoPath
    try {
        cargo test -p bimcraft-ui-egui --features egui_kittest/wgpu --lib capture_industrial_operation_choices --config 'profile.dev.package.bimcraft-core.opt-level=0' --config 'profile.dev.debug=0' -- --test-threads=4
        $exitCode=$LASTEXITCODE
    } finally { Pop-Location }
} finally {
    [IO.File]::WriteAllBytes($testRoot,$original)
    foreach($path in $captureOnlyMocks.Keys) { [IO.File]::WriteAllBytes($path,$captureOnlyMocks[$path]) }
    if (Test-Path -LiteralPath $tempModule) { Remove-Item -LiteralPath $tempModule }
}
exit $exitCode
