using Microsoft.AspNetCore;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Configuration;
using System.IO;

namespace QCExtensions.Server
{
	public class Program
	{
		public static void Main(string[] args)
		{
			BuildWebHost(args).Run();
		}

		public static IWebHost BuildWebHost(string[] args)
		{
			return WebHost.CreateDefaultBuilder(args)
				.ConfigureAppConfiguration(c => {
					c
						.SetBasePath(Directory.GetCurrentDirectory())
						.AddJsonFile("appsettings.json")
						.AddEnvironmentVariables()
						.AddMySqlDatabaseUrlConnectionString();
				})
				.UseStartup<Startup>()
				.Build();
		}
	}
}