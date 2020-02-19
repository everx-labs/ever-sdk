G_giturl = "git@github.com:tonlabs/TON-SDK.git"
G_gitcred = 'TonJenSSH'
G_container = "atomxy/empty-ton-sdk-js:20191128"
C_PROJECT = "NotSet"
C_COMMITER = "NotSet"
C_HASH = "NotSet"
C_TEXT = "NotSet"
G_binversion = "NotSet"
G_tsnj_build = true
G_tsnj_deploy = true

DiscordURL = "https://discordapp.com/api/webhooks/496992026932543489/4exQIw18D4U_4T0H76bS3Voui4SyD7yCQzLP9IRQHKpwGRJK1-IFnyZLyYzDmcBKFTJw"

def checkAndCreateBranch(ton_client_url) {
    ton_repo_name = ton_client_url.substring(ton_client_url.lastIndexOf('/') + 1, ton_client_url.lastIndexOf('.') )
    tmp_dir = "tmp/${ton_repo_name}-version"
    ton_client_js_path = "git+ssh://git@github.com/tonlabs/ton-client-js.git#${G_binversion}-rc"
    sh (script:  """
        mkdir -pv $tmp_dir
        git clone $ton_client_url $tmp_dir
        cd $tmp_dir
        if (git ls-remote --heads --exit-code $ton_client_url ${GIT_BRANCH})
        then
            echo "Branch name ${GIT_BRANCH} in $ton_client_url already exists."
        else
            git checkout -b ${GIT_BRANCH}

            case ${ton_repo_name} in
            "ton-client-rs")
                sed -i 's@^version\\s*=\\s*"[0-9]*\\.[0-9]*\\.[0-9]*"@version = "${G_binversion}"@g' Cargo.toml
                ;;

            "ton-client-js")
                sed -i 's@"version"\\s*:\\s*"[0-9]*\\.[0-9]*\\.[0-9]*"@"version": "${G_binversion}"@g' package.json
                ;;
            esac
            git add .
            git commit -m 'automate Jenkins branch ${GIT_BRANCH}'
            git push --set-upstream origin ${GIT_BRANCH}
            echo "Branch ${GIT_BRANCH} in $ton_client_url was created."
        fi
    """)
}


pipeline {
    agent none
    tools {nodejs "Node12.8.0"}
    options {
        buildDiscarder logRotator(artifactDaysToKeepStr: '', artifactNumToKeepStr: '', daysToKeepStr: '', numToKeepStr: '10')
        disableConcurrentBuilds()
        parallelsAlwaysFailFast()
    }
    triggers {
        upstream(
            upstreamProjects: 'ton-labs-vm/master,SDK/ton-labs-abi/master',
            threshold: hudson.model.Result.SUCCESS
        )
    }
    stages {
        stage('Initialize') {
            agent any
            steps {
                script {
                    G_gitproject = G_giturl.substring(15,G_giturl.length()-4)
                    G_gitproject_dir = G_gitproject.substring(8, G_gitproject.length())
                    C_TEXT = sh (script: "git show -s --format=%s ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    C_AUTHOR = sh (script: "git show -s --format=%an ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    C_COMMITER = sh (script: "git show -s --format=%cn ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    C_HASH = sh (script: "git show -s --format=%h ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    C_PROJECT = G_giturl.substring(15,G_giturl.length()-4)
                    C_GITURL = sh (script: "echo ${GIT_URL}",returnStdout: true).trim()
                    C_GITCOMMIT = sh (script: "echo ${GIT_COMMIT}", \
                        returnStdout: true).trim()
                    G_binversion = sh (script: 'cat ton_client/client/Cargo.toml | grep -Eo "^version = \\".*\\"" | grep -Eo "[0-9\\.]*"', \
                        returnStdout: true).trim()
                }
                echo "Version: ${G_binversion}."
                echo "Branch: ${GIT_BRANCH}"
                echo "Possible RC: ${G_binversion}-rc"
            }
        }
        stage('Check branch in ton-client-js,ton-client-rs') {
            agent any
            when {
                expression {
                    GIT_BRANCH == "${G_binversion}-rc"
                }
            }
            steps {
                script {
                    sshagent (credentials: [G_gitcred]) {
                        checkAndCreateBranch("git@github.com:tonlabs/ton-client-js.git")
                        checkAndCreateBranch('git@github.com:tonlabs/ton-client-rs.git')
                    }
                }
            }
        }
        stage('Building...') {
            failFast true
            parallel {
                stage('Client linux') {
                    agent {
                        docker {
                            image G_container
                        }
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh 'rustc --version'
                                sh 'cargo --version'
                            }
                        }
                        stage('Build') {
                            steps {
                                dir('ton_client/client') {
                                    sshagent([G_gitcred]) {
                                        sh 'node build.js'
                                    }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/client/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('Client macOS') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh 'rustc --version'
                                sh 'cargo --version'
                            }
                        }
                        stage('Build') {
                            steps {
                                dir('ton_client/client') {
                                    sshagent([G_gitcred]) {
                                        sh 'node build.js'
                                    }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/client/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('Client Windows') {
                    agent {
                        label "Win"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                bat 'rustc --version'
                                bat 'cargo --version'
                            }
                        }
                        stage('Build') {
                            steps {
                                dir('ton_client/client') {
                                    sshagent([G_gitcred]) {
                                        bat 'node build.js'
                                    }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/client/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('react-native-ios') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('ton_client/platforms/ton-client-react-native') {
                                        sh 'node build.js --ios'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/platforms/ton-client-react-native/output') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('react-native-android') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('ton_client/platforms/ton-client-react-native') {
                                        sh 'node build.js --android'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/platforms/ton-client-react-native/output') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('node-js for iOS') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('ton_client/platforms/ton-client-node-js') {
                                        sh 'node build.js'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/platforms/ton-client-node-js/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('node-js for Windows') {
                    agent {
                        label "Win"
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                bat '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('ton_client/platforms/ton-client-node-js') {
                                        bat 'node build.js'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/platforms/ton-client-node-js/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('node-js for Linux') {
                    agent {
                        docker {
                            image G_container
                        }
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('ton_client/platforms/ton-client-node-js') {
                                        sh 'node build.js'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/platforms/ton-client-node-js/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
                stage('web') {
                    agent {
                        docker {
                            image G_container
                        }
                    }
                    stages {
                        stage('Report versions') {
                            steps {
                                sh '''
                                rustc --version
                                cargo --version
                                '''
                            }
                        }
                        stage('Build') {
                            environment {
                                X86_64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR='/usr/lib/x86_64-linux-gnu'
                                X86_64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR='/usr/include/openssl'
                                X86_64_UNKNOWN_LINUX_GNU_OPENSSL_DIR='/usr/bin/openssl'
                                OPENSSL_DIR='/usr/bin/openssl'
                            }
                            steps {
                                echo 'Install...'
                                sshagent([G_gitcred]) {
                                    dir('ton_client/platforms/ton-client-web') {
                                        sh 'npm install'
                                    }
                                }
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    dir('ton_client/platforms/ton-client-web') {
                                        sh 'node build.js'
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when {
                                expression {
                                    GIT_BRANCH == 'master' || GIT_BRANCH == "${G_binversion}-rc"
                                }
                            }
                            steps {
                                dir('ton_client/platforms/ton-client-web/bin') {
                                    script {
                                        withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                            identity = awsIdentity()
                                            s3Upload \
                                                bucket: 'sdkbinaries.tonlabs.io', \
                                                includePathPattern:'*.gz', workingDir:'.'
                                            }
                                    }
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
					post {
						cleanup {script{cleanWs notFailBuild: true}}
					}
				}
            }
        }
    }
}
