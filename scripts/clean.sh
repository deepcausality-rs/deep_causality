#
# Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
#

# bin/bash
set -o errexit
set -o nounset
set -o pipefail

delete_folder (){
if [ -d "$1" ];
then
    rm -r "$1"
fi
}

delete_folder 01_release
delete_folder gen
delete_folder out

mkdir -p gen
mkdir -p gen/exp/btc_01
mkdir -p gen/exp/btc_02
mkdir -p gen/exp/btc_03
mkdir -p gen/exp/btc_04
mkdir -p gen/exp/btc_05
