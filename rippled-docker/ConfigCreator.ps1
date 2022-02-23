$n = $args[0]
$path = "..\config"
$dbpath = "..\db"
$logpath = "..\logs"

$validator_path = -join($path, "\validators.txt")
$validator_public_keys = Get-Content -path $validator_path -Tail $n

$cluster_seed_path = -join($path, "\cluster_seeds.json")
$cluster_seed_file = (Get-Content -path $cluster_seed_path | ConvertFrom-Json)

$cluster_seeds = @()
For ($i=1; $i -le $n; $i++) {
  $cluster_seeds += $cluster_seed_file.nodes[$i-1].validation_seed
  # empty db folder
  Remove-Item (-join($dbpath, "\validator_", $i, "\*")) -Exclude .gitkeep -Recurse
  # empty log folder
  Remove-Item (-join($logpath, "\validator_", $i, "\*")) -Exclude .gitkeep -Recurse
}

$rippled_cfg_base = Get-Content -path (-join($path, "\rippled.cfg"))

For ($i=0; $i -lt $n; $i++) {
  Write-Output (-join("Creating config files for ripple", ($i+1)))

  # rippled.cfg
  $config_contents = (-join(($rippled_cfg_base[0..33] | out-string), $cluster_seeds[$i], ($rippled_cfg_base[34..$rippled_cfg_base.Length] | out-string)))
  $config_contents | Out-File -Encoding ascii -FilePath (-join($path, "\validator_" , ($i+1), "\rippled.cfg"))

  # validators.txt
  $own_pub_key = $validator_public_keys[$i]
  $validator_contents = (-join("[validators]`n", ($validator_public_keys[0..($i)] -ne $own_pub_key | Out-String), ($validator_public_keys[($i)..($n-1)] -ne $own_pub_key | Out-String)))
  $validator_contents | Out-File -Encoding ascii -FilePath (-join($path, "\validator_" , ($i+1), "\validators.txt"))
}
