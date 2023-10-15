@echo off
setlocal enabledelayedexpansion

REM Check if psql is installed
@REM where psql > nul 2>nul
@REM if %errorlevel% neq 0 (
@REM     echo Error: psql is not installed. >&2
@REM     exit /b 1
@REM )

REM Check if sqlx is installed
where sqlx > nul 2>nul
if %errorlevel% neq 0 (
    echo Error: sqlx is not installed. >&2
    echo Use: >&2
    echo cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres >&2
    echo to install it. >&2
    exit /b 1
)

REM Check if a custom user has been set, otherwise default to 'postgres'
set "DB_USER=%POSTGRES_USER:postgres%"
if "%DB_USER%" == "%POSTGRES_USER%" set "DB_USER=postgres"

REM Check if a custom password has been set, otherwise default to 'password'
set "DB_PASSWORD=%POSTGRES_PASSWORD:password%"
if "%DB_PASSWORD%" == "%POSTGRES_PASSWORD%" set "DB_PASSWORD=password"

REM Check if a custom database name has been set, otherwise default to 'newsletter'
set "DB_NAME=%POSTGRES_DB:newsletter%"
if "%DB_NAME%" == "%POSTGRES_DB%" set "DB_NAME=newsletter"

REM Check if a custom port has been set, otherwise default to '5432'
set "DB_PORT=%POSTGRES_PORT:5432%"
if "%DB_PORT%" == "%POSTGRES_PORT%" set "DB_PORT=5432"

REM Launch postgres using Docker
set IMAGE_NAME=postgres
set CONTAINER_NAME=email_news_con

REM Check if container already exists before starting a new instance
docker ps -q -f "name=%CONTAINER_NAME%" | findstr /R ".*" > nul
if %errorlevel% equ 0 (
    echo Container is already running. > nul
) else (
    REM If the container doesn't exist, delete any existing container with the same name (if any)
    docker rm %CONTAINER_NAME% 2>nul    

    docker run -e POSTGRES_USER=%DB_USER% -e POSTGRES_PASSWORD=%DB_PASSWORD% -e POSTGRES_DB=%DB_NAME% -p 5432:5432 -d --name %CONTAINER_NAME% %IMAGE_NAME%
    
    REM Wait for Postgres to start up
    timeout 5 > nul
)
REM Increased maximum number of connections for testing purposes.

REM Keep pinging Postgres until it's up and running
:retry
@REM psql -h localhost -U %DB_USER% -p %DB_PORT% -d postgres -c "\q" > nul 2>nul
@REM if errorlevel 1 (
@REM     echo Postgres is stil unavaiable - sleeping
@REM     ping -n 2 127.0.0.1 > nul
@REM     goto retry
@REM ) else (
@REM     echo Postgres is up and running on port %DB_PORT%!
@REM )

REM Initialize DATABASE_URL environment variable for sqlx
set  "DATABASE_URL=postgres://%DB_USER%:%DB_PASSWORD%@localhost:%DB_PORT%/%DB_NAME%"
echo %DATABASE_URL%

REM Create database in Postgres
sqlx database create
echo sqlx: Database created successfully.

REM Run initial migrations
sqlx migrate run

endlocal