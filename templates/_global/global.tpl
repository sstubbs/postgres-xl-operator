{{ $app_name := printf "%s-%s" .cleaned_release_name .cluster.cleaned_name }}

{{ define "global_labels" }}
    app.kubernetes.io/name: {{ .cluster.cleaned_name }}
    helm.sh/chart: {{ .cleaned_name }}-{{ .version }}
    app.kubernetes.io/managed-by: {{ .release_service }}
    app.kubernetes.io/instance: {{ .cleaned_release_name }}
    app.kubernetes.io/version: {{ .cluster.values.image.version }}
{{- if .cluster.values.extra_labels }}
{{ range .cluster.values.extra_labels -}}
{{ .name | indent 4 }}: {{ .content }}
{{ end -}}
{{- end }}
{{- end -}}