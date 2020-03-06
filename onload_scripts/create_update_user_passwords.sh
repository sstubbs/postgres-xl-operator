for file in "${PASSWORD_SECRET_MOUNT_PATH}/"*; do
  USER="${file##*/}"
  PASSWORD=$(cat "${file}")

  if [ "${USER}" != "${PGUSER}" ]; then
    psql -c "CREATE USER ${USER} WITH PASSWORD '${PASSWORD}';"
  fi
  psql -c "ALTER USER ${USER} WITH PASSWORD '${PASSWORD}';"
done
