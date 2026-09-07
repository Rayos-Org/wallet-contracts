$commits = @(
    @{
        message = "build(policy): initialize policy contract crate"
        files = @("contracts/policy/Cargo.toml")
    },
    @{
        message = "feat(policy): add storage types and error codes"
        files = @("contracts/policy/src/errors.rs", "contracts/policy/src/storage.rs")
    },
    @{
        message = "feat(policy): implement rolling spend limits"
        files = @("contracts/policy/src/spend_limits.rs")
    },
    @{
        message = "feat(policy): implement scoped session keys"
        files = @("contracts/policy/src/session_keys.rs")
    },
    @{
        message = "feat(policy): implement target contract allow-lists"
        files = @("contracts/policy/src/allow_lists.rs")
    },
    @{
        message = "feat(policy): implement N-of-M guardian recovery"
        files = @("contracts/policy/src/guardians.rs")
    },
    @{
        message = "feat(policy): expose public policy entrypoints"
        files = @("contracts/policy/src/lib.rs")
    },
    @{
        message = "test(policy): add comprehensive unit and proptests"
        files = @("contracts/policy/src/test.rs", "contracts/policy/test_snapshots/")
    },
    @{
        message = "build(factory): initialize factory contract crate"
        files = @("contracts/factory/Cargo.toml")
    },
    @{
        message = "feat(factory): implement deterministic wallet deployment"
        files = @("contracts/factory/src/storage.rs", "contracts/factory/src/lib.rs")
    },
    @{
        message = "test(factory): add factory deployment tests"
        files = @("contracts/factory/src/test.rs", "contracts/factory/test_snapshots/")
    },
    @{
        message = "test: add multi-contract integration flows"
        files = @("tests/")
    },
    @{
        message = "ci: configure github actions workflow and issue templates"
        files = @(".github/")
    },
    @{
        message = "chore: add testnet deployment and binding scripts"
        files = @("scripts/")
    },
    @{
        message = "docs: add migration guide and initial architecture"
        files = @("MIGRATION.md", "wallet-contracts-ARCHITECTURE.md")
    },
    @{
        message = "docs: add comprehensive developer guides and threat model"
        files = @("docs/")
    },
    @{
        message = "docs: finalize open-source repository documentation"
        files = @("README.md", "LICENSE", "CONTRIBUTING.md", "SECURITY.md", "CODE_OF_CONDUCT.md", "CHANGELOG.md")
    }
)

Write-Host "Continuing spaced commits and pushes from policy contract..." -ForegroundColor Cyan
Write-Host "Remaining commits: $($commits.Length)" -ForegroundColor Cyan

foreach ($i in 0..($commits.Length - 1)) {
    $commit = $commits[$i]
    Write-Host "`n[$($i + 1)/$($commits.Length)] Adding: $($commit.files -join ', ')" -ForegroundColor Yellow

    foreach ($file in $commit.files) {
        if (Test-Path $file) {
            git add $file
        }
    }

    $staged = git diff --cached --name-only
    if (-not $staged) {
        Write-Host "Nothing to commit for this step, skipping." -ForegroundColor DarkGray
        continue
    }

    git commit -m $commit.message
    Write-Host "Pushing: $($commit.message)" -ForegroundColor Yellow
    git push origin main

    if ($i -lt ($commits.Length - 1)) {
        $waitTime = Get-Random -Minimum 65 -Maximum 110
        Write-Host "Sleeping $waitTime seconds before next push..." -ForegroundColor DarkGray
        Start-Sleep -Seconds $waitTime
    }
}

Write-Host "`n✅ All commits pushed successfully!" -ForegroundColor Green
