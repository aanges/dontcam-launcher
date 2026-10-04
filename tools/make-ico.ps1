$files = @('src-tauri/icons/32x32.png','src-tauri/icons/128x128.png','src-tauri/icons/256x256.png')
$sizes = @(32,128,256)
$blobs = @()
foreach ($f in $files) { $blobs += ,[IO.File]::ReadAllBytes($f) }
$n = $blobs.Count
$ms = New-Object IO.MemoryStream
$bw = New-Object IO.BinaryWriter($ms)
$bw.Write([uint16]0)
$bw.Write([uint16]1)
$bw.Write([uint16]$n)
$offset = 6 + 16 * $n
for ($i = 0; $i -lt $n; $i++) {
  $s = $sizes[$i]
  if ($s -ge 256) { $w = [byte]0 } else { $w = [byte]$s }
  $bw.Write($w)
  $bw.Write($w)
  $bw.Write([byte]0)
  $bw.Write([byte]0)
  $bw.Write([uint16]1)
  $bw.Write([uint16]32)
  $bw.Write([uint32]$blobs[$i].Length)
  $bw.Write([uint32]$offset)
  $offset += $blobs[$i].Length
}
foreach ($bb in $blobs) { $bw.Write($bb) }
$bw.Flush()
[IO.File]::WriteAllBytes('src-tauri/icons/icon.ico', $ms.ToArray())
$bw.Close()
$ms.Close()
$b = [IO.File]::ReadAllBytes('src-tauri/icons/icon.ico')
Write-Host ("ico ok, size {0}, count {1}" -f $b.Length, $b[4])
