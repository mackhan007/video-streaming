{{- define "streaming.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "streaming.labels" -}}
app.kubernetes.io/name: {{ include "streaming.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}

{{/* Which S3 backend is enabled: "minio" or "localstack". */}}
{{- define "streaming.s3Backend" -}}
{{- if and .Values.minio.enabled .Values.localstack.enabled -}}
{{- fail "enable only one of minio.enabled / localstack.enabled" -}}
{{- else if .Values.minio.enabled -}}minio
{{- else if .Values.localstack.enabled -}}localstack
{{- else -}}{{- fail "enable one of minio.enabled / localstack.enabled" -}}
{{- end -}}
{{- end }}

{{/* In-cluster host:port of the S3 API. */}}
{{- define "streaming.s3Upstream" -}}
{{- if eq (include "streaming.s3Backend" .) "minio" -}}
{{ include "streaming.name" . }}-minio:{{ .Values.service.minioPort }}
{{- else -}}
{{ include "streaming.name" . }}-localstack:{{ .Values.service.localstackPort }}
{{- end -}}
{{- end }}

{{/* Browser/host-facing S3 endpoint (the port scripts/k8s-pf.sh forwards). */}}
{{- define "streaming.s3PublicEndpoint" -}}
{{- if eq (include "streaming.s3Backend" .) "minio" -}}
http://127.0.0.1:{{ .Values.service.minioPort }}
{{- else -}}
http://127.0.0.1:{{ .Values.service.localstackPort }}
{{- end -}}
{{- end }}
