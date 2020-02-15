{{- $component := "envs" -}}

apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ $app_name }}-{{ $component }}
  labels:
    app.kubernetes.io/component: {{ $component }}
{{- template "global_labels" . }}
data:
  PORT_WAIT_INTERVAL: "1"
  PORT_WAIT_TRIES: "60"
  PORT_WAIT_TIMEOUT: "1"
  LOGGING_PREFIX: "PGXL:HELM:"
  RESET_DB: "false"

  # the postgres user to use for connections and the root
  # db user.
  PGUSER: "{{ .cluster.values.config.postgres_user }}"

  # the user authentication type
  AUTH_TYPE: "{{ .cluster.values.security.postgres_auth_type }}"

{{- if .cluster.values.replication.master_name -}}
  # if this is a standby it will have a master
  MASTER_NAME: "{{ printf "%s-%s" .cleaned_release_name .cluster.values.replication.master_name }}"
{{ end }}

  # the wal archive directory. Can be overriden.
  WAL_ARCHIVE_PATH: "{{ .cluster.values.homedir }}/wal_archive/{{ .cluster.values.wal.archive.version }}"

{{- if .cluster.values.envs -}}
  # Added envs. These will not affect the db operation.
{{ range .cluster.values.envs -}}
{{ .name | indent 2 }}: "{{ .content }}"
{{ end -}}
{{ end }}

  PG_GTM_HOST: "{{ $app_name }}-svc-gtm"
  PG_GTM_PORT: "{{ .cluster.values.config.managers_port }}"
  PG_GTM_COORDINATOR_SVC_HOST: "{{ $app_name }}-svc"
  PG_PORT: "{{ .cluster.values.config.postgres_port }}"
  PG_HOST: "0.0.0.0"
  PGDATA: "{{ .cluster.values.homedir }}/storage/data"
  STORAGE_MOUNT_PATH: "{{ .cluster.values.homedir }}/storage"
  EXTERNAL_SERVICE: "{{ $app_name }}-svc"
  HOSTALIASES: "/config/host_aliases"

  GTM_BASENAME: "{{ $app_name }}-gtm"
  GTM_SERVICE: "{{ $app_name }}-svc-gtm"
  PROXY_COUNT: "{{ .cluster.values.proxies.count }}"
  PROXY_SERVICE: "{{ $app_name }}-svc-pxy"
  PROXY_ENABLED:  "{{ .cluster.values.proxies.enabled }}"
  PROXY_BASENAME: "{{ $app_name }}-pxy"
  PROXY_THREAD_COUNT: "{{ .cluster.values.proxies.thread_count }}"
  COORDINATOR_COUNT: "{{ .cluster.values.coordinators.count }}"
  DATANODE_COUNT: "{{ .cluster.values.datanodes.count }}"
  DATANODE_BASENAME: "{{ $app_name }}-dn"
  COORDINATOR_BASENAME: "{{ $app_name }}-crd"
  DATANODE_SERVICE: "{{ $app_name }}-svc-dn"
  COORDINATOR_SERVICE: "{{ $app_name }}-svc-crd"

{{- if .cluster.values.override_envs -}}
  # Added envs, these may affect pod operation.
{{ range .cluster.values.override_envs -}}
{{ .name | indent 2 }}: "{{ .content }}"
{{ end -}}
{{- end -}}