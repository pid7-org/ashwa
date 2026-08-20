output "instance_id" {
  value       = aws_instance.bench.id
  description = "The EC2 Instance ID"
}

output "public_ip" {
  value       = aws_instance.bench.public_ip
  description = "The public IP of the benchmark instance"
}

output "private_key_path" {
  value       = local_file.private_key.filename
  description = "Path to the local SSH private key"
}

output "ssh_command" {
  value       = "ssh -o StrictHostKeyChecking=no -i ${local_file.private_key.filename} ubuntu@${aws_instance.bench.public_ip}"
  description = "SSH command to connect to the instance"
}
