using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Server.Infrastructure.Services.Hosted
{
	public abstract class RepeatingBackgroundService : BackgroundService
	{
		protected abstract Task<bool> ExecuteRepeatedlyAsync(CancellationToken stoppingToken);

		protected sealed override async Task ExecuteAsync(CancellationToken stoppingToken)
		{
			bool keepRepeating = true;
			while (!stoppingToken.IsCancellationRequested && keepRepeating)
			{
				keepRepeating = await ExecuteRepeatedlyAsync(stoppingToken);
			}
		}
	}
}
