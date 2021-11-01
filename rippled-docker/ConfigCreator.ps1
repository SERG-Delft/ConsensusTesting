$total = 5
$n = $args[0]
$connected = $args[1]
$path = "..\config"
$dbpath = "..\db"
$logpath = "..\logs"

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
  # empty log folder
  Remove-Item (-join($logpath, "\validator_", $i, "\*")) -Exclude .gitkeep -Recurse
}

$rippled_cfg_base = Get-Content -path (-join($path, "\rippled.cfg"))

For ($i=0; $i -lt $n; $i++) {
  Write-Output (-join("Creating config files for ripple", ($i+1)))

  # rippled.cfg
  $own_ip = $rippled_cfg_base[(48 + $i)]
  if ($connected -eq "1") {
    $ips_fixed = (-join(($rippled_cfg_base[48..(48+$i)] -ne $own_ip | Out-String), ($rippled_cfg_base[(48+$i)..(48+$n-1)] -ne $own_ip | Out-String)))
    $cluster = ""
  } else {
    $ips_fixed = $rippled_cfg_base[53]
    $cluster = (-join(($rippled_cfg_base[54 ..55] | out-string), $cluster_seeds[$i], ($rippled_cfg_base[56..58] | Out-String)))
  }
  $config_contents = (-join(($rippled_cfg_base[0..33] | out-string), $validator_tokens[$i],
        ($rippled_cfg_base[34..47] | Out-String), $ips_fixed,
        $cluster,
        ($rippled_cfg_base[59..$rippled_cfg_base.Length] | out-string)))
  $config_contents | Out-File -FilePath (-join($path, "\validator_" , ($i+1), "\rippled.cfg"))

  # validators.txt
  $own_pub_key = $validator_public_keys[$i]
  $validator_contents = (-join("[validators]`n", ($validator_public_keys[0..($i)] -ne $own_pub_key | Out-String), ($validator_public_keys[($i)..($n-1)] -ne $own_pub_key | Out-String)))

  # Write-Output (-join(($validator_contents | Out-String), "`n"))
  $validator_contents | Out-File -FilePath (-join($path, "\validator_" , ($i+1), "\validators.txt"))
}
