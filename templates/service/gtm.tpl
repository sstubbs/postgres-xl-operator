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
    app.kubernetes.io/instance: {{ .cleaned_release_name }}
    app.kubernetes.io/name: {{ .cluster.cleaned_name }}
    app.kubernetes.io/component: {{ $component }}