#!/bin/bash

missing_prereq=0
if ! command -v docker &> /dev/null
then
    echo "docker is not installed, but is required"
    missing_prereq=1
fi
if ! command -v mysql &> /dev/null
then
    echo "mysql-client is not installed, but is required"
    missing_prereq=1
fi
if ! command -v liquibase &> /dev/null
then
    echo "liquibase is not installed, but is required"
    missing_prereq=1
fi

# If any command was not installed, exit with status 1
if [ $missing_prereq -eq 1 ]; then
    exit 1
fi

# Cleanup existing database.  Maybe put this behind an option.
echo "Cleaning up existing database..."
docker stop planting_life_db > /dev/null
docker rm planting_life_db > /dev/null
rm -rf `pwd`/db/data

# Putting data in a subdirectory, to leave a reasonable place for config
echo ""
echo "Making directory..."
mkdir -p db/data

# Start container, mapping db/data to the data directory
echo ""
echo "Pulling mariadb docker image..."
docker pull mariadb

echo ""
echo "Starting mariadb..."
docker run \
  --name planting_life_db \
  -p 3306:3306 \
  -v  `pwd`/db/data:/var/lib/mysql \
  -e MYSQL_ROOT_PASSWORD=dev_password1235 \
  --security-opt seccomp=unconfined \
  -d \
  mariadb > /dev/null

# It will take a few seconds for the server to start.
# Watch for the ready message.
until docker logs planting_life_db 2>&1 | grep -q "mariadbd: ready for connections"; do
  sleep 1
done

# Waiting a few more seconds seems to be needed.
# Without this I get: ERROR 2013 (HY000): Lost connection to MySQL server 
#                     at 'reading initial communication packet
sleep 10 

echo ""
echo "Setup db/users..."
mysql -h 127.0.0.1 -u root --password=dev_password1235 -e "
   
  -- Create the planting_life database
  CREATE DATABASE planting_life;

  -- Create the admin user, for creating tables/etc.
  -- Intended to be used by migrations.
  CREATE USER 'planting_life_admin'@'%' IDENTIFIED BY 'DevAdminPassword';
  GRANT ALL PRIVILEGES 
    ON planting_life.* 
    TO 'planting_life_admin'@'%' 
    WITH GRANT OPTION;

  -- Create the normal user, for CRUD operations on existing tables.
  -- Intended to be used by the web app.
  CREATE USER 'planting_life_user'@'%' IDENTIFIED BY 'DevUserPassword';
  GRANT SELECT, INSERT, UPDATE, DELETE 
    ON planting_life.* 
    TO 'planting_life_user'@'%';

  FLUSH PRIVILEGES;
"

#TODO: The above command shouldn't directly have the password in it, try
#      something like: https://stackoverflow.com/questions/20751352/suppress-warning-messages-using-mysql-from-within-terminal-but-password-written
