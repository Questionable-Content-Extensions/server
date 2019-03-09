using AngleSharp.Html.Parser;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;
using System;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Server.Infrastructure.Services.Hosted
{
	public class BackgroundNewsUpdatingService : RepeatingBackgroundService
	{
		private static HttpClient m_httpClient = new HttpClient();

		private const string QCComicUrlBase = "https://questionablecontent.net/view.php?comic=";
		private static readonly int TaskDelayTime = (int)TimeSpan.FromSeconds(5).TotalMilliseconds;

		private readonly IServiceProvider _provider;
		private readonly INewsUpdater _newsUpdater;
		private readonly ILogger<BackgroundNewsUpdatingService> _logger;

		public BackgroundNewsUpdatingService(
			IServiceProvider serviceProvider,
			INewsUpdater newsUpdater,
			ILogger<BackgroundNewsUpdatingService> logger)
		{
			_provider = serviceProvider;
			_newsUpdater = newsUpdater;
			_logger = logger;
		}

		protected override async Task<bool> ExecuteRepeatedlyAsync(CancellationToken stoppingToken)
		{
			var updateEntries = _newsUpdater.GetPendingUpdateEntries();
			_logger.LogDebug($"There are #{updateEntries.Count} news updates pending.");

			if (updateEntries.Count > 0)
			{
				_logger.LogInformation($"Running background news update...");
				using (var scope = _provider.CreateScope())
				{
					var context = scope.ServiceProvider.GetRequiredService<DomainDbContext>();
					foreach (var comicId in updateEntries)
					{
						var comicExists = await context.Comics.ExistsAsync(comicId);
						if (!comicExists)
						{
							_logger.LogInformation($"Cannot update news for comic #{comicId}; comic data does not yet exist.");
							continue;
						}

						var newsEntity = await context.News.SingleOrDefaultAsync(n => n.ComicId == comicId);
						if (newsEntity != null && !newsEntity.IsOutdated)
						{
							_logger.LogInformation($"News for comic #{comicId} is not outdated.");
							continue;
						}

						_logger.LogInformation($"Fetching news in the background for comic #{comicId}...");
						string newsText = null;
						try
						{
							newsText = await FetchNewsForAsync(comicId);
						}
						catch (Exception e)
						{
							_logger.LogError(e, "Fetching news failed");
							continue;
						}

						if (newsText == null)
						{
							continue;
						}

						if (newsEntity == null)
						{
							// New news
							newsEntity = new News
							{
								ComicId = comicId,
								LastUpdated = DateTime.UtcNow,
								UpdateFactor = 1,
								NewsText = newsText
							};
							context.News.Add(newsEntity);
						}
						else
						{
							// Old news. Compare news text with old.
							if (newsEntity.NewsText == newsText)
							{
								_logger.LogInformation($"News text for comic #{comicId} is the same. Increasing update factor.");
								newsEntity.UpdateFactor += 0.5;
							}
							else
							{
								_logger.LogInformation($"News text for comic #{comicId} has changed. Resetting update factor and updating text.");
								newsEntity.UpdateFactor = 1;
								newsEntity.NewsText = newsText;
							}
							newsEntity.LastUpdated = DateTime.UtcNow;
							context.News.Update(newsEntity);
						}
					}
					_logger.LogInformation($"Saving any changes to the news to the database.");
					await context.SaveChangesAsync();

					_newsUpdater.RemovePendingUpdateEntries(updateEntries);
				}
			}

			_logger.LogDebug($"Waiting #{TaskDelayTime} before checking again.");
			await Task.Delay(TaskDelayTime, stoppingToken);
			return true;
		}

		private async Task<string> FetchNewsForAsync(int comic)
		{
			var response = await m_httpClient.GetAsync($"{QCComicUrlBase}{comic}");
			if (!response.IsSuccessStatusCode)
			{
				_logger.LogWarning($"Could not fetch news for #{comic}, got HTTP status {response.StatusCode}");
				return null;
			}
			var qcPage = await response.Content.ReadAsStringAsync();

			if (string.IsNullOrEmpty(qcPage))
			{
				_logger.LogWarning($"Could not fetch news for #{comic}, got empty response");
				return null;
			}

			var parser = new HtmlParser();
			var document = parser.ParseDocument(qcPage);

			var newsElement = document.GetElementById("news");
			if (newsElement == null)
			{
				_logger.LogWarning($"Could not fetch news for #{comic}, couldn't find #news element");
				return null;
			}

			// Clean up crud at the beginning of the newspost
			if (newsElement.FirstElementChild?.NodeName == "B")
			{
				newsElement.RemoveChild(newsElement.FirstElementChild);
			}
			while (newsElement.FirstChild != null && (newsElement.FirstChild.NodeName == "BR" || (newsElement.FirstChild.NodeName == "#text" && string.IsNullOrEmpty(newsElement.FirstChild.NodeValue.Trim()))))
			{
				newsElement.RemoveChild(newsElement.FirstChild);
			}

			return newsElement.InnerHtml
				.Replace("\r", "")
				.Replace("\n", "")
				.Replace("<br />", "<br>")
				.Replace("<br/>", "<br>")
				.Replace("<br>", "\n")
				.Trim();
		}
	}
}