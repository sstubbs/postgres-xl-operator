apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ $app_name }}-scripts
  labels:
    app.kubernetes.io/component: scripts
{{- template "global_labels" . }}
data:
{{ range .cluster.scripts -}}
{{ indent 2 .name }}: |
{{ indent 4 .script }}
{{ end -}}