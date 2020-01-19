{{- $component := "gtm" -}}

apiVersion: v1
kind: Service
metadata:
  name: {{ $app_name }}-svc-{{ $component }}
  labels:
    app.kubernetes.io/component: {{ $component }}
{{- template "global_labels" . }}
spec:
  clusterIP: None
  ports:
  - port: {{ .cluster.values.config.managers_port }}
    protocol: TCP
    targetPort: {{ .cluster.values.config.managers_port }}
    name: {{ $component }}
  selector:
    app.kubernetes.io/component: {{ $component }}
{{ range .cluster.selector_labels -}}
{{ .name | indent 4 }}: {{ .content }}
{{ end }}