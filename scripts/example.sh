# SPDX-License-Identifier: MIT
# Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
set -o errexit
set -o nounset
set -o pipefail

# Bash Select (Make Menu) https://linuxize.com/post/bash-select/

echo ""
echo "--------------------------------"
echo "Select example to run: "
echo "--------------------------------"
echo ""

select opt in  csm ctx dtx smoking quit;
do
  case $opt in
    csm)
      echo "Selected example: CSM (Causal State Machine)"
      command cargo run --release --bin example-csm
      break
      ;;
    ctx)
      echo "Selected example: CTX (Context)"
      command cargo run --release --bin example-ctx
      break
      ;;
    dtx)
      echo "Selected example: DTX (Dynamic Context)"
      command cargo run --release --bin example-dtx
      break
      ;;
    smoking)
      echo "Selected example: SMOKING (Smoking)"
       command cargo run --release --bin example-smoking
      break
      ;;
    quit)
      echo "Exiting!"
      exit 0
      ;;
    *)
      echo "Invalid option $REPLY"
      ;;
  esac
done
