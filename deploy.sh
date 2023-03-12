#!/bin/sh

# Instantly exits our script whenever an error occurs
set -e

# Pipe our environmental SSH key variable into a file
mkdir -p $HOME/.ssh
echo "${deploy_key}" > $HOME/.ssh/deploy_key
chmod 600 $HOME/.ssh/deploy_key # SSH keys need to be readonly

# Where to deploy our site on our server
target="/home/${deploy_user}/qcext-server/staging"

# The actual deployment
sh -c "rsync -azvh -e 'ssh -i $HOME/.ssh/deploy_key -o StrictHostKeyChecking=no' qcext-server.tar.bz2 ${deploy_user}@${deploy_target}:${target}"

# Run update script
sh -c "ssh -i $HOME/.ssh/deploy_key -o StrictHostKeyChecking=no ${deploy_user}@${deploy_target} 'cd ${target}; ./update.sh'"

# Remove our deploy_key again since it's no longer needed
rm $HOME/.ssh/deploy_key
