{{- $component := "scripts" -}}

apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ $app_name }}-{{ $component }}
  labels:
    app.kubernetes.io/component: {{ $component }}
{{- template "global_labels" . }}
data:
  # Load scripts from the scripts directory.
{{ range .cluster.scripts -}}
{{ .name | indent 2 }}: |-
{{ .content | indent 4 }}
{{ end -}}