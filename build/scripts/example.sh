#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

select opt in  cate csm dbn ethos granger starter scm rcm quit;
do
  case $opt in

    cate)
      echo "Selected example: CATE (Conditional Average Treatment Effect)"
      command cargo run --release --bin example-cate
      break
      ;;

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

    ethos)
      echo "Selected example: ethos (CSM with Effect Ethos)"
      command cargo run --release --bin example_effect_ethos
      break
      ;;

    scm)
      echo "Selected example: scm (Structured Causal Model)"
      command cargo run --release --bin example-scm
      break
      ;;

    starter)
      echo "Selected example: Starter (Starter)"
      command cargo run --release --bin starter_example
      break
      ;;

    granger)
      echo "Selected example: Granger (Granger causality)"
      command cargo run --release --bin example-granger
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
