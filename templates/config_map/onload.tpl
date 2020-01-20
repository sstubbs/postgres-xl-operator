{{- $component := "onload" -}}

# Main condition.
{{ if .cluster.values.on_load.enabled }}

apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ $app_name }}-{{ $component }}
  labels:
    app.kubernetes.io/component: {{ $component }}
{{- template "global_labels" . }}
data:
{{ range $i, $v := .cluster.values.on_load.startup }}
  startup_{{ $i }}_{{ $v.name }}: |-
{{ $v.content | indent 4 }}
{{ end }}
{{ range $i, $v := .cluster.values.on_load.init }}
  init_{{ $i }}_{{ $v.name }}: |-
{{ $v.content | indent 4 }}
{{ end }}

# End of main condition.
{{- end }}