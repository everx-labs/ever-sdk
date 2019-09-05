G_giturl = "git@github.com:tonlabs/TON-SDK.git"
G_gitcred = 'TonJenSSH'
G_container = "atomxy/empty-ton-sdk-js:20190827"
C_PROJECT = "NotSet"
C_COMMITER = "NotSet"
C_HASH = "NotSet"
C_TEXT = "NotSet"
G_binversion = "NotSet"
G_tsnj_build = true
G_tsnj_deploy = true

DiscordURL = "https://discordapp.com/api/webhooks/496992026932543489/4exQIw18D4U_4T0H76bS3Voui4SyD7yCQzLP9IRQHKpwGRJK1-IFnyZLyYzDmcBKFTJw"
def getVar(Gvar) {
    return Gvar
}

pipeline {
    agent none
    tools {nodejs "Node12.8.0"}
    options {
        buildDiscarder logRotator(artifactDaysToKeepStr: '', artifactNumToKeepStr: '', daysToKeepStr: '', numToKeepStr: '10')
        disableConcurrentBuilds()
        parallelsAlwaysFailFast()
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
                echo "Version: ${getVar(G_binversion)}."
            }
        }
        stage('Build client') {
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
                stage('Build client') {
                    steps {
                        sh '''cd ton_client/client
                            cargo update
                            cargo build'''
                    }
                }
            }
        }
        stage('Building ton-sdk-node-js') {
            failFast true
            parallel {
                stage('node-js for iOS') {
                    agent {
                        label "ios"
                    }
                    stages {
                        stage('Versions') {
                            steps {
                                sh '''
                                cd ton_client/ton-sdk-node-js
                                rustc --version'
                                cargo --version
                                '''
                            }
                        }
                        stage('Build') {
                            steps {
                                echo 'Build ...'
                                sshagent([G_gitcred]) {
                                    sh 'node build.js'
                                }
                            }
                            post {
                                failure {
                                    script { G_tsnj_build = false }
                                }
                            }
                        }
                        stage('Deploy') {
                            when { branch 'master' }
                            steps {
                                sh 'cd bin'
                                withAWS(credentials: 'CI_bucket_writer', region: 'eu-central-1') {
                                    identity = awsIdentity()
                                    s3Upload \
                                        bucket: 'sdkbinaries.tonlabs.io', \
                                        path:'.', includePathPattern:'**/*', workingDir:'.', excludePathPattern:'**/*.svg'
                                    }
                            }
                            post {
                                failure {
                                    script { G_tsnj_deploy = false }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}