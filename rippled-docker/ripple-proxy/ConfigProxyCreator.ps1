$total = 5
$n = $args[0]
$path = "..\config"
$dbpath = "..\db"

$validator_path = -join($path, "\validators.txt")
$validator_public_keys = Get-Content -path $validator_path -Tail $total

$cluster_seed_path = -join($path, "\cluster_seeds.json")
$cluster_seed_file = (Get-Content -path $cluster_seed_path | ConvertFrom-Json)

$validator_tokens = @()
$cluster_seeds = @()
For ($i=1; $i -le $n; $i++) {
  $validator_token_path = -join($path, "\validator-keys_", $i, ".json")
  $validator_tokens += (Get-Content -path $validator_token_path | ConvertFrom-Json).token
  $cluster_seeds += $cluster_seed_file.nodes[$i-1].validation_seed
  # empty db folder
  Remove-Item (-join($dbpath, "\validator_", $i, "\*")) -Exclude .gitkeep -Recurse
}

$rippled_cfg_base = Get-Content -path (-join($path, "\rippled-proxy-base.cfg"))

For ($i=0; $i -lt $n; $i++) {
  Write-Output (-join("Creating config files for ripple", ($i+1)))

  # rippled.cfg
  $cluster = (-join(($rippled_cfg_base[54 ..55] | out-string), $cluster_seeds[$i], ($rippled_cfg_base[56..58] | Out-String)))

  $config_contents = (-join(($rippled_cfg_base[0..33] | out-string), $validator_tokens[$i],
        ($rippled_cfg_base[34..50] | Out-String), $cluster,
        ($rippled_cfg_base[54..$rippled_cfg_base.Length] | out-string)))
  $config_contents | Out-File -FilePath (-join($path, "\validator_" , ($i+1), "\rippled.cfg"))

  # validators.txt
  $own_pub_key = $validator_public_keys[$i]
  $validator_contents = (-join("[validators]`n", ($validator_public_keys[0..($i)] -ne $own_pub_key | Out-String), ($validator_public_keys[($i)..($n-1)] -ne $own_pub_key | Out-String)))
  $validator_contents | Out-File -FilePath (-join($path, "\validator_" , ($i+1), "\validators.txt"))
}
