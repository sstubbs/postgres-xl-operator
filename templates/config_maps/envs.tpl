apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ .cluster.name }}-envs
  labels:
    app.kubernetes.io/component: env-config
{{- template "global_labels" . }}
data:
  PORT_WAIT_INTERVAL: "1"
  PORT_WAIT_TRIES: "60"
  PORT_WAIT_TIMEOUT: "1"
  LOGGING_PREFIX: "PGXL:HELM:"
  RESET_DB: "false"

  # the postgres user to use for connections and the root
  # db user.
  PGUSER: {{ .cluster.values.config.postgres_user }}

  # the user authentication type
  AUTH_TYPE: {{ .cluster.values.security.postgres_auth_type }}

  # the wal archive directory. Can be overriden.
  WAL_ARCHIVE_PATH: "{{ .cluster.values.homedir }}/wal_archive/{{ .cluster.values.wal.archive.version }}"

  # Added envs. These will not affect the db operation.
{{ if .cluster.values.envs }}
{{ indent 2 .cluster.values.envs }}
{{ end }}

  PG_GTM_HOST: {{ .cluster.cleaned_name }}-svc-gtm
  PG_GTM_PORT: "{{ .cluster.values.config.managers_port }}"
  PG_GTM_COORDINATOR_SVC_HOST: {{ .cluster.cleaned_name }}-svc
  PG_PORT: "{{ .cluster.values.config.postgres_port }}"
  PG_HOST: "0.0.0.0"
  PGDATA: "{{ .cluster.values.homedir }}/storage/data"
  STORAGE_MOUNT_PATH: "{{ .cluster.values.homedir }}/storage"
  EXTERNAL_SERVICE: "{{ .cluster.cleaned_name }}-svc"
  HOSTALIASES: "/config/host_aliases"

  GTM_BASENAME: "{{ .cluster.cleaned_name }}-gtm"
  GTM_SERVICE: "{{ .cluster.cleaned_name }}-svc-gtm"
  PROXY_COUNT: "{{ .cluster.values.proxies.count }}"
  PROXY_SERVICE: {{ .cluster.cleaned_name }}-svc-pxy
  PROXY_ENABLED:  "{{ .cluster.values.proxies.enabled }}"
  PROXY_BASENAME: "{{ .cluster.cleaned_name }}-pxy"
  PROXY_THREAD_COUNT: "{{ .cluster.values.proxies.thread_count }}"
  COORDINATOR_COUNT: "{{ .cluster.values.coordinators.count }}"
  DATANODE_COUNT: "{{ .cluster.values.datanodes.count }}"
  DATANODE_BASENAME: "{{ .cluster.cleaned_name }}-dn"
  COORDINATOR_BASENAME: "{{ .cluster.cleaned_name }}-crd"
  DATANODE_SERVICE: {{ .cluster.cleaned_name }}-svc-dn
  COORDINATOR_SERVICE: {{ .cluster.cleaned_name }}-svc-crd

  # Added envs, these may affect pod operation.
{{ if .cluster.values.override_envs }}
{{ indent 2 .cluster.values.override_envs }}
{{ end }}