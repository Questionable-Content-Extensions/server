using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Configuration.Memory;
using System;
using System.Collections.Generic;

namespace QCExtensions.Server.Infrastructure.Configuration
{
	public class MySqlDatabaseUrlConnectionStringConfigurationProvider : ConfigurationProvider
	{
		public MySqlDatabaseUrlConnectionStringConfigurationProvider(string connectionName)
		{
			var databaseUrl = Environment.GetEnvironmentVariable("DATABASE_URL");
			if (string.IsNullOrEmpty(databaseUrl))
			{
				return;
			}

			Uri mysqlUri;
			string username;
			string password;
			string host;
			string database;
			int port;
			try
			{
				mysqlUri = new Uri(databaseUrl);

				var usernamePassword = mysqlUri.UserInfo.Split(':');
				username = usernamePassword[0];
				password = usernamePassword[1];
				host = mysqlUri.Host;
				port = mysqlUri.Port == -1 ? 5432 : mysqlUri.Port;
				database = mysqlUri.AbsolutePath.Substring(1);
			}
			catch (Exception)
			{
				Console.Error.WriteLine("DATABASE_URL is in an invalid format");
				return;
			}

			// "Server=localhost;Database=ef;User=root;Password=123456;"
			Data = new Dictionary<string, string>
			{
				{
					$"ConnectionStrings:{connectionName}",
					$"User={username};Password={password};Server={host};Port={port};Database={database};" //Integrated Security=true;Pooling=true;"
				}
			};
		}
	}
}
