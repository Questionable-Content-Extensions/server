using Microsoft.Extensions.Configuration;
using QCExtensions.Server.Infrastructure.Configuration;

namespace Microsoft.Extensions.Configuration
{
	public static class MySqlDatabaseUrlConfigurationBuilderExtensions
	{
		public static IConfigurationBuilder AddMySqlDatabaseUrlConnectionString(this IConfigurationBuilder configurationBuilder, string connectionName = "Default")
		{
			return configurationBuilder
				.Add(new MySqlDatabaseUrlConnectionStringConfigurationSource(connectionName));
		}
	}
}
