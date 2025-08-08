#
# SPDX-License-Identifier: MIT
# Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail

# Bash Select (Make Menu) https://linuxize.com/post/bash-select/

echo ""
echo "--------------------------------"
echo "Select example to run: "
echo "--------------------------------"
echo "csm: Causal state machine"
echo "starter: Simplest getting started example"
echo "smoking: Simple causal model without  Context"
echo "--------------------------------"
echo ""

select opt in  csm dbn starter smoking rcm quit;
do
  case $opt in

    csm)
      echo "Selected example: CSM (Causal State Machine)"
      command cargo run --release --bin example-csm
      break
      ;;

    dbn)
      echo "Selected example: DBN (Dynamic Bayesian Network)"
      command cargo run --release --bin example-dbn
      break
      ;;

    starter)
      echo "Selected example: Starter (Starter)"
      command cargo run --release --bin example-starter
      break
      ;;

    smoking)
      echo "Selected example: SMOKING (Smoking)"
       command cargo run --release --bin example-smoking
      break
      ;;

    rcm)
      echo "Selected example: RCM (Rubin Causal Model)"
       command cargo run --release --bin example-rcm
      break
      ;;
    quit)
      echo "Exiting!"
      exit 0
      ;;

    *)
      echo "Invalid option $REPLY"
      echo "Exiting!"
      exit 0
      ;;
  esac
done
