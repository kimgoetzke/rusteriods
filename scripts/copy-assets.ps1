# Define the directories
$sourceDirectory = "./assets"
$destinationDirectory = "./www/public/assets"

Write-Host "Copying assets from $sourceDirectory to $destinationDirectory"

# Remove the destination directory, if it exists
if (Test-Path -Path $destinationDirectory) {
    Remove-Item -Path $destinationDirectory -Recurse -Force
    Write-Host "Removed the existing destination directory: $destinationDirectory"
}

# Create the destination directory, if it doesn't exist
if (-Not (Test-Path -Path $destinationDirectory))
{
    New-Item -ItemType Directory -Path $destinationDirectory
    Write-Host "Created the destination directory: $destinationDirectory"
}

# Copy all files and directories, excluding ./assets/github/*
Get-ChildItem -Path $sourceDirectory -Recurse | Where-Object { $_.FullName -notlike "*\github\*" } | ForEach-Object {
    $destPath = $_.FullName.Replace((Get-Item $sourceDirectory).FullName, (Get-Item $destinationDirectory).FullName)
    Write-Host "Copying $_ to $destPath"
    if ($_.PSIsContainer)
    {
        # Create the directory, if it doesn't exist
        if (-Not (Test-Path -Path $destPath))
        {
            Write-Host "Creating directory: $destPath"
            New-Item -ItemType Directory -Path $destPath
        }
    }
    else
    {
        # Copy the file
        Copy-Item -Path $_.FullName -Destination $destPath -Force
    }
}

Write-Host "DONE!"