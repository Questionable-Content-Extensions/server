using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Configuration.Memory;
using System.Collections.Generic;

namespace QCExtensions.Server.Infrastructure.Configuration
{
	public class MySqlDatabaseUrlConnectionStringConfigurationSource : IConfigurationSource
	{
		private string _connectionName;

		public MySqlDatabaseUrlConnectionStringConfigurationSource(string connectionName)
		{
			_connectionName = connectionName;
		}

		public IConfigurationProvider Build (IConfigurationBuilder builder)
		{
			return new MySqlDatabaseUrlConnectionStringConfigurationProvider(_connectionName);
		}
	}
}
