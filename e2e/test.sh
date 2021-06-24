#!/usr/bin/env bash
set -ex

## run from repo main dir

export DOCKER_BUILDKIT=1

kubectl delete pod cdl-e2e || echo ""

docker build -f Dockerfile.e2e -t cdl-e2e:latest --build-arg ENV=DEV .


kubectl apply -f ./e2e/pod.yml
sleep 5
kubectl logs cdl-e2e --follow 
[[ -z `kubectl get pods -o=jsonpath="{.items[?(.status.containerStatuses[0].state.terminated.exitCode!=0)]['metadata.name']}"` ]]
kubectl delete pod cdl-e2e

