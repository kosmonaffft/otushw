#!/bin/bash
set -e

export PGPASSWORD=postgres

echo "Waiting for PostgreSQL to be ready..."
until pg_isready -U postgres -h localhost; do
  echo "Coordinator is unavailable - sleeping"
  sleep 2
done

echo "Citus extension should already be installed, verifying..."
psql -U postgres -h localhost -c "SELECT * FROM pg_extension WHERE extname = 'citus';"

max_retries=30
retry_count=0

echo "Checking if worker1 already exists..."
worker1_exists=$(psql -U postgres -h localhost -t -c "SELECT COUNT(*) FROM citus_get_active_worker_nodes() WHERE node_name = 'worker1';" 2>/dev/null || echo "0")
worker1_exists=$(echo $worker1_exists | xargs)

if [ "$worker1_exists" = "0" ] || [ -z "$worker1_exists" ]; then
  echo "Adding worker1..."
  while [ $retry_count -lt $max_retries ]; do
    if psql -U postgres -h localhost -c "SELECT * FROM citus_add_node('worker1', 5432);" 2>/dev/null; then
      echo "Added worker1"
      break
    fi
    echo "Waiting for worker1 to be ready... (attempt $((retry_count + 1))/$max_retries)"
    sleep 2
    retry_count=$((retry_count + 1))
  done

  if [ $retry_count -eq $max_retries ]; then
    echo "Failed to add worker1 after $max_retries attempts"
    exit 1
  fi
else
  echo "worker1 already exists, skipping"
fi

retry_count=0
echo "Checking if worker2 already exists..."
worker2_exists=$(psql -U postgres -h localhost -t -c "SELECT COUNT(*) FROM citus_get_active_worker_nodes() WHERE node_name = 'worker2';" 2>/dev/null || echo "0")
worker2_exists=$(echo $worker2_exists | xargs)

if [ "$worker2_exists" = "0" ] || [ -z "$worker2_exists" ]; then
  echo "Adding worker2..."
  while [ $retry_count -lt $max_retries ]; do
    if psql -U postgres -h localhost -c "SELECT * FROM citus_add_node('worker2', 5432);" 2>/dev/null; then
      echo "Added worker2"
      break
    fi
    echo "Waiting for worker2 to be ready... (attempt $((retry_count + 1))/$max_retries)"
    sleep 2
    retry_count=$((retry_count + 1))
  done

  if [ $retry_count -eq $max_retries ]; then
    echo "Failed to add worker2 after $max_retries attempts"
    exit 1
  fi
else
  echo "worker2 already exists, skipping"
fi

echo "Verifying cluster nodes..."
psql -U postgres -h localhost -c "SELECT * FROM citus_get_active_worker_nodes();"

echo "Citus cluster initialization complete!"
