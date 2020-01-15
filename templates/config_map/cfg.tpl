apiVersion: v1
metadata:
  name: {{ $app_name }}-cfg
  labels:
    app.kubernetes.io/component: cfg
{{- template "global_labels" . }}
data:
  config_append_internal_global: |
    # applies only on startup.
    listen_addresses = '*'

  config_append_gtm: |
    # applies only on startup.
    log_min_messages = {{ upper .cluster.values.config.log_level }}
{{- if .cluster.values.config.append.gtm }}
{{ .cluster.values.config.append.gtm | indent 4 }}
{{- end }}

  config_append_proxy: |
    # applies only on startup.
    log_min_messages = {{ upper .cluster.values.config.log_level }}
{{- if .cluster.values.config.append.proxy }}
{{ .cluster.values.config.append.proxy | indent 4 }}
{{- end }}

  config_append_datanode: |
    # applies only on startup.
    log_min_messages = {{ lower .cluster.values.config.log_level }}
{{- if .cluster.values.wal.archive.enable }}
    # archive the data
    archive_mode = on
    # archive command.
    archive_command = '/scripts/wal_archive %p %f'
{{- end }}
{{- if .cluster.values.config.append.datanode }}
{{ .cluster.values.config.append.datanode | indent 4 }}
{{- end }}

  config_append_coordinator: |
    # applies only on startup.
    log_min_messages = {{ lower .cluster.values.config.log_level }}
{{- if .cluster.values.config.append.coordinator }}
{{ .cluster.values.config.append.coordinator | indent 4 }}
{{- end }}

  host_aliases: |
    # list of hosts to alias, for datanode and coordinators.
    # These short names are required by pg since the host
    # name is truncated by the Create Node sql query.
    local-alias-gtm {{ $app_name }}-gtm-0.{{ $app_name }}-svc-gtm

{{- range $i := until (int .cluster.values.datanodes.count) }}
    local-alias-dn-{{ $i }} {{ $app_name }}-dn-{{ $i }}.{{ $app_name }}-svc-dn
{{- end }}