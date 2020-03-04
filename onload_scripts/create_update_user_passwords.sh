for file in "${PASSWORD_SECRET_MOUNT_PATH}/"*; do
  USER="${file##*/}"
  PASSWORD=$(cat "${file}")
  psql -c "CREATE USER ${USER} WITH PASSWORD '${PASSWORD}';" || true
  psql -c "ALTER USER ${USER} WITH PASSWORD '${PASSWORD}';" || true
done


