{{- $component := "cfg" -}}

apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ $app_name }}-{{ $component }}
  labels:
    app.kubernetes.io/component: {{ $component }}
{{- template "global_labels" . }}
data:
  config_append_internal_global: |
    # applies only on startup.
    listen_addresses = '*'

  config_append_gtm: |
    # applies only on startup.
    log_min_messages = {{ upper .cluster.values.config.log_level }}
{{- if .cluster.values.config.append.gtm }}
{{ range .cluster.values.config.append.gtm -}}
{{ .name | indent 4 }} = {{ .content }}
{{ end -}}
{{- end }}

  config_append_proxy: |
    # applies only on startup.
    log_min_messages = {{ upper .cluster.values.config.log_level }}
{{- if .cluster.values.config.append.proxy }}
{{ range .cluster.values.config.append.proxy -}}
{{ .name | indent 4 }} = {{ .content }}
{{ end -}}
{{- end }}

  config_append_datanode: |
    # applies only on startup.
    log_min_messages = {{ lower .cluster.values.config.log_level }}
{{- if .cluster.values.config.append.datanode }}
{{ range .cluster.values.config.append.datanode -}}
{{ .name | indent 4 }} = {{ .content }}
{{ end -}}
{{- end }}

  config_append_coordinator: |
    # applies only on startup.
    log_min_messages = {{ lower .cluster.values.config.log_level }}
{{- if .cluster.values.config.append.coordinator }}
{{ range .cluster.values.config.append.coordinator -}}
{{ .name | indent 4 }} = {{ .content }}
{{ end -}}
{{- end }}

  host_aliases: |
    # list of hosts to alias, for datanode and coordinators.
    # These short names are required by pg since the host
    # name is truncated by the Create Node sql query.
    local-alias-gtm {{ $app_name }}-gtm-0.{{ $app_name }}-svc-gtm
{{- range (until .cluster.values.datanodes.count) }}
    local-alias-dn-{{ . }} {{ $app_name }}-dn-{{ . }}.{{ $app_name }}-svc-dn
{{- end }}
{{- range (until .cluster.values.coordinators.count) }}
    local-alias-crd-{{ . }} {{ $app_name }}-crd-{{ . }}.{{ $app_name }}-svc-crd
{{- end }}
{{- range (until .cluster.values.proxies.count) }}
    local-alias-pxy-{{ . }} {{ $app_name }}-pxy-{{ . }}.{{ $app_name }}-svc-pxy
{{- end }}


