apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ .cluster.name }}-envs
  labels:
    app.kubernetes.io/component: standard-config
{{- template "global_labels" . }}
data:
  PORT_WAIT_INTERVAL: "1"