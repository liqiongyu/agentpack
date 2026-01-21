Output docs/assets/demo.gif

Require agentpack
Require jq

Set Shell "bash"
Set Theme "Dracula"
Set FontSize 16
Set Width 720
Set Height 432
Set Framerate 5
Set PlaybackSpeed 1.8
Set TypingSpeed 40ms

Type "tmp=$(mktemp -d)"
Enter
Type "export HOME=$tmp/home"
Enter
Type "export AGENTPACK_HOME=$tmp/agentpack_home"
Enter
Type "mkdir -p $HOME $AGENTPACK_HOME $tmp/workspace"
Enter
Type "cp -R docs/examples/minimal_repo $tmp/repo"
Enter
Type "cd $tmp/workspace"
Enter

Type "agentpack --repo $tmp/repo update"
Enter
Type "agentpack --repo $tmp/repo preview --diff"
Enter
Type "snapshot_id=$(agentpack --repo $tmp/repo --json --yes deploy --apply | jq -r '.data.snapshot_id')"
Enter
Type "echo snapshot_id=$snapshot_id"
Enter
Type "agentpack --repo $tmp/repo status"
Enter
Type "agentpack --repo $tmp/repo --yes rollback --to $snapshot_id"
Enter

Sleep 1s
